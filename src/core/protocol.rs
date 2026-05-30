use std::io::{Read, Write};
use std::net::TcpStream;

enum TransportStream {
    Plain(TcpStream),
    Tls(Box<native_tls::TlsStream<TcpStream>>),
}

impl Read for TransportStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            TransportStream::Plain(s) => s.read(buf),
            TransportStream::Tls(s) => s.read(buf),
        }
    }
}

impl Write for TransportStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            TransportStream::Plain(s) => s.write(buf),
            TransportStream::Tls(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            TransportStream::Plain(s) => s.flush(),
            TransportStream::Tls(s) => s.flush(),
        }
    }
}

pub fn pkt_line_encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return b"0000".to_vec();
    }
    let len = data.len() + 4;
    let header = format!("{:04x}", len);
    let mut result = Vec::with_capacity(header.len() + data.len());
    result.extend_from_slice(header.as_bytes());
    result.extend_from_slice(data);
    result
}

pub fn pkt_line_flush() -> Vec<u8> {
    b"0000".to_vec()
}

pub fn pkt_line_decode(line: &[u8]) -> Option<Vec<u8>> {
    if line.len() < 4 {
        return None;
    }
    let len_str = std::str::from_utf8(&line[..4]).ok()?;
    let len = u16::from_str_radix(len_str, 16).ok()? as usize;
    if len == 0 {
        return Some(vec![]);
    }
    if len < 4 || line.len() < len {
        return None;
    }
    Some(line[4..len].to_vec())
}

pub fn parse_refs_data(data: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(data);
    let mut refs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(pkt) = parse_pkt_line(line) {
            if pkt.is_empty() {
                continue;
            }
            let pkt_str = String::from_utf8_lossy(&pkt);
            let parts: Vec<&str> = pkt_str.splitn(2, ' ').collect();
            if parts.len() >= 2 && parts[0].len() == 40 {
                refs.push((parts[0].to_string(), parts[1].to_string()));
            }
        }
    }
    refs
}

fn parse_pkt_line(line: &str) -> Option<Vec<u8>> {
    if line.len() < 4 {
        return None;
    }
    let len = u16::from_str_radix(&line[..4], 16).ok()? as usize;
    if len == 0 {
        return Some(vec![]);
    }
    if len < 4 || line.len() < len {
        return None;
    }
    Some(line.as_bytes()[4..len].to_vec())
}

pub type ObjectList = Vec<(String, Vec<u8>)>;

pub fn parse_packfile(data: &[u8]) -> Result<ObjectList, Box<dyn std::error::Error>> {
    if data.len() < 12 {
        return Err("Packfile too short".into());
    }
    if &data[..4] != b"PACK" {
        return Err("Invalid packfile signature".into());
    }
    let _version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let num_objects = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;

    struct RawObj {
        obj_start: usize,
        obj_type: u8,
        decompressed: Vec<u8>,
        consumed: usize,
    }

    let mut raw_objects: Vec<RawObj> = Vec::with_capacity(num_objects);
    let mut pos = 12;

    for _ in 0..num_objects {
        if pos >= data.len() {
            break;
        }
        let obj_start = pos;
        let byte = data[pos];
        pos += 1;
        let obj_type = (byte >> 4) & 0x07;
        let mut _decompressed_size = (byte & 0x0F) as usize;
        let mut shift = 4;
        let mut more_bytes = byte & 0x80 != 0;
        while more_bytes {
            if pos >= data.len() {
                break;
            }
            let next_byte = data[pos];
            pos += 1;
            _decompressed_size |= ((next_byte & 0x7F) as usize) << shift;
            shift += 7;
            more_bytes = next_byte & 0x80 != 0;
        }

        let (decompressed, consumed) = match obj_type {
            1..=4 | 6..=7 => {
                let (content, n) = crate::core::compression::decompress_stream(&data[pos..])?;
                (content, n)
            }
            _ => {
                pos += 1;
                continue;
            }
        };

        pos += consumed;
        raw_objects.push(RawObj { obj_start, obj_type, decompressed, consumed: consumed + 1 });
    }

    let mut resolved: Vec<(String, Vec<u8>, usize)> = Vec::with_capacity(num_objects);

    for raw in &raw_objects {
        if raw.obj_type >= 1 && raw.obj_type <= 4 {
            let type_str = match raw.obj_type {
                1 => "commit",
                2 => "tree",
                3 => "blob",
                4 => "tag",
                _ => unreachable!(),
            };
            let header = format!("{} {}\0", type_str, raw.decompressed.len());
            let mut obj_data = Vec::with_capacity(header.len() + raw.decompressed.len());
            obj_data.extend_from_slice(header.as_bytes());
            obj_data.extend_from_slice(&raw.decompressed);
            let sha1 = crate::core::hash::hash_bytes(&obj_data);
            resolved.push((sha1, obj_data, raw.obj_start));
        }
    }

    for raw in &raw_objects {
        match raw.obj_type {
            6 => {
                let offset_data = &data[raw.obj_start + 1..];
                let (neg_offset, _skip) = decode_varint(offset_data);
                let base_pos = if raw.obj_start >= neg_offset as usize {
                    raw.obj_start - neg_offset as usize
                } else {
                    continue;
                };
                if let Some((_, base_obj, _)) = resolved.iter().find(|&&(_, _, o)| o == base_pos) {
                    let base_content = &base_obj[base_obj.iter().position(|&b| b == 0).unwrap_or(0) + 1..];
                    if let Ok(resolved_content) = apply_delta(base_content, &raw.decompressed) {
                        let type_str = detect_type_from_header(base_obj);
                        let header = format!("{} {}\0", type_str, resolved_content.len());
                        let mut obj_data = Vec::with_capacity(header.len() + resolved_content.len());
                        obj_data.extend_from_slice(header.as_bytes());
                        obj_data.extend_from_slice(&resolved_content);
                        let sha1 = crate::core::hash::hash_bytes(&obj_data);
                        resolved.push((sha1, obj_data, raw.obj_start));
                    }
                }
            }
            7 => {
                let ref_sha1_start = raw.obj_start + 1;
                if ref_sha1_start + 20 <= data.len() {
                    let ref_bytes = &data[ref_sha1_start..ref_sha1_start + 20];
                    let base_sha1 = bytes_to_hex(ref_bytes);
                    if let Some((_, base_obj, _)) = resolved.iter().find(|(s, _, _)| s == &base_sha1) {
                        let base_content = &base_obj[base_obj.iter().position(|&b| b == 0).unwrap_or(0) + 1..];
                        if let Ok(resolved_content) = apply_delta(base_content, &raw.decompressed) {
                            let type_str = detect_type_from_header(base_obj);
                            let header = format!("{} {}\0", type_str, resolved_content.len());
                            let mut obj_data = Vec::with_capacity(header.len() + resolved_content.len());
                            obj_data.extend_from_slice(header.as_bytes());
                            obj_data.extend_from_slice(&resolved_content);
                            let sha1 = crate::core::hash::hash_bytes(&obj_data);
                            resolved.push((sha1, obj_data, raw.obj_start));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(resolved.into_iter().map(|(s, d, _)| (s, d)).collect())
}

fn decode_varint(data: &[u8]) -> (u64, usize) {
    let mut value: u64 = 0;
    let mut shift = 0;
    let mut pos = 0;
    while pos < data.len() {
        let byte = data[pos];
        pos += 1;
        let lower_7 = (byte & 0x7F) as u64;
        value |= lower_7 << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
        value += 1 << shift;
    }
    (value, pos)
}

fn apply_delta(base: &[u8], delta: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (source_size, mut pos) = decode_varint(delta);
    let (target_size, consumed) = decode_varint(&delta[pos..]);
    pos += consumed;

    let _ = source_size;
    let _ = target_size;

    let mut result = Vec::new();
    while pos < delta.len() {
        let cmd = delta[pos];
        pos += 1;

        if cmd & 0x80 == 0 {
            let len = (cmd & 0x7F) as usize + 1;
            if pos + len > delta.len() {
                break;
            }
            result.extend_from_slice(&delta[pos..pos + len]);
            pos += len;
        } else {
            let mut offset: usize = 0;
            let mut size: usize = 0;
            let mut byte_count = 0;

            if cmd & 0x01 != 0 { offset |= (delta[pos] as usize) << (8 * byte_count); pos += 1; byte_count += 1; }
            if cmd & 0x02 != 0 { offset |= (delta[pos] as usize) << (8 * byte_count); pos += 1; byte_count += 1; }
            if cmd & 0x04 != 0 { offset |= (delta[pos] as usize) << (8 * byte_count); pos += 1; byte_count += 1; }
            if cmd & 0x08 != 0 { offset |= (delta[pos] as usize) << (8 * byte_count); pos += 1; }

            byte_count = 0;
            if cmd & 0x10 != 0 { size |= (delta[pos] as usize) << (8 * byte_count); pos += 1; byte_count += 1; }
            if cmd & 0x20 != 0 { size |= (delta[pos] as usize) << (8 * byte_count); pos += 1; byte_count += 1; }
            if cmd & 0x40 != 0 { size |= (delta[pos] as usize) << (8 * byte_count); pos += 1; }

            if size == 0 { size = 0x10000; }
            if offset + size > base.len() {
                break;
            }
            result.extend_from_slice(&base[offset..offset + size]);
        }
    }

    Ok(result)
}

fn detect_type_from_header(obj_data: &[u8]) -> &str {
    if let Some(null_pos) = obj_data.iter().position(|&b| b == 0) {
        let header = std::str::from_utf8(&obj_data[..null_pos]).unwrap_or("");
        header.split(' ').next().unwrap_or("blob")
    } else {
        "blob"
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub struct HttpTransport {
    host: String,
    port: u16,
    path: String,
    use_ssl: bool,
}

impl HttpTransport {
    pub fn from_url(url_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed = url::Url::parse(url_str)?;
        let host = parsed
            .host_str()
            .ok_or("Invalid URL: missing host")?
            .to_string();
        let use_ssl = parsed.scheme() == "https";
        let port = parsed.port().unwrap_or(if use_ssl { 443 } else { 80 });
        let path = parsed.path().to_string();
        Ok(HttpTransport {
            host,
            port,
            path,
            use_ssl,
        })
    }

    pub fn url_for_service(&self, service: &str) -> String {
        format!("{}/info/refs?service={}", self.path, service)
    }

    pub fn url_for_upload_pack(&self) -> String {
        format!("{}/git-upload-pack", self.path)
    }

    pub fn url_for_receive_pack(&self) -> String {
        format!("{}/git-receive-pack", self.path)
    }

    fn connect(&self) -> Result<TransportStream, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.host, self.port);
        let tcp = TcpStream::connect(&addr)?;
        tcp.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;

        if self.use_ssl {
            let connector = native_tls::TlsConnector::builder()
                .build()?;
            let tls = connector.connect(&self.host, tcp)?;
            Ok(TransportStream::Tls(Box::new(tls)))
        } else {
            Ok(TransportStream::Plain(tcp))
        }
    }

    fn http_get(
        &self,
        path: &str,
    ) -> Result<(u16, Vec<u8>), Box<dyn std::error::Error>> {
        let mut stream = self.connect()?;
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: agit/0.1.0\r\nAccept: */*\r\nConnection: close\r\n\r\n",
            path, self.host
        );
        stream.write_all(request.as_bytes())?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        parse_http_response(&response)
    }

    fn http_post(
        &self,
        path: &str,
        content_type: &str,
        body: &[u8],
    ) -> Result<(u16, Vec<u8>), Box<dyn std::error::Error>> {
        let mut stream = self.connect()?;
        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: agit/0.1.0\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            path, self.host, content_type, body.len()
        );
        stream.write_all(request.as_bytes())?;
        stream.write_all(body)?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        parse_http_response(&response)
    }

    pub fn discover_refs(&self) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let url = self.url_for_service("git-upload-pack");
        let (status, body) = self.http_get(&url)?;
        if status != 200 {
            return Err(format!("HTTP {}: refs discovery failed", status).into());
        }
        Ok(parse_refs_data(&body))
    }

    pub fn clone_full(
        &self,
        want_sha1: &str,
    ) -> Result<ObjectList, Box<dyn std::error::Error>> {
        let mut body = Vec::new();
        body.extend_from_slice(&pkt_line_encode(
            format!("want {} multi_ack_detailed no-done side-band-64k thin-pack ofs-delta agent=agit/0.1.0\n", want_sha1).as_bytes(),
        ));
        body.extend_from_slice(&pkt_line_flush());
        body.extend_from_slice(&pkt_line_encode(b"done\n"));
        body.extend_from_slice(&pkt_line_flush());

        let (status, response) =
            self.http_post(&self.url_for_upload_pack(), "application/x-git-upload-pack-request", &body)?;
        if status != 200 {
            return Err(format!("HTTP {}: upload-pack failed", status).into());
        }

        let pack_start = find_pack_start(&response);
        if pack_start >= response.len() {
            return Err("No packfile in response".into());
        }

        parse_packfile(&response[pack_start..])
    }

    pub fn fetch_objects(
        &self,
        wants: &[String],
        haves: &[String],
    ) -> Result<ObjectList, Box<dyn std::error::Error>> {
        let mut body = Vec::new();
        for want in wants {
            body.extend_from_slice(&pkt_line_encode(
                format!("want {} multi_ack_detailed no-done side-band-64k thin-pack ofs-delta agent=agit/0.1.0\n", want).as_bytes(),
            ));
        }
        for have in haves {
            body.extend_from_slice(&pkt_line_encode(
                format!("have {}\n", have).as_bytes(),
            ));
        }
        body.extend_from_slice(&pkt_line_flush());
        body.extend_from_slice(&pkt_line_encode(b"done\n"));
        body.extend_from_slice(&pkt_line_flush());

        let (status, response) =
            self.http_post(&self.url_for_upload_pack(), "application/x-git-upload-pack-request", &body)?;
        if status != 200 {
            return Err(format!("HTTP {}: upload-pack failed", status).into());
        }

        let pack_start = find_pack_start(&response);
        if pack_start >= response.len() {
            return Err("No packfile in response".into());
        }

        parse_packfile(&response[pack_start..])
    }

    pub fn push_pack(
        &self,
        ref_update: &str,
        pack_data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut body = Vec::new();
        let report_cap = "report-status side-band-64k agent=agit/0.1.0";
        body.extend_from_slice(&pkt_line_encode(
            format!("{} {}\0{}\n", ref_update, ref_update, report_cap).as_bytes(),
        ));
        body.extend_from_slice(&pkt_line_flush());
        body.extend_from_slice(pack_data);

        let (status, response) =
            self.http_post(&self.url_for_receive_pack(), "application/x-git-receive-pack-request", &body)?;
        if status != 200 {
            return Err(format!("HTTP {}: receive-pack failed: {}", status, String::from_utf8_lossy(&response)).into());
        }

        Ok(())
    }
}

fn parse_http_response(data: &[u8]) -> Result<(u16, Vec<u8>), Box<dyn std::error::Error>> {
    let text = String::from_utf8_lossy(data);
    let lines: Vec<&str> = text.split("\r\n").collect();
    if lines.is_empty() {
        return Err("Empty HTTP response".into());
    }

    let status_line = lines[0];
    let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err("Invalid HTTP status line".into());
    }
    let status = parts[1].parse::<u16>().unwrap_or(500);

    let header_end = data
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(data.len());

    let body = data[header_end..].to_vec();
    Ok((status, body))
}

fn find_pack_start(data: &[u8]) -> usize {
    for i in 0..data.len().saturating_sub(4) {
        if &data[i..i + 4] == b"PACK" {
            return i;
        }
    }
    data.len()
}
