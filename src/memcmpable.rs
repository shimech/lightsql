use std::cmp;

const ESCAPE_LENGTH: usize = 8;

pub fn encoded_size(len: usize) -> usize {
    (len + (ESCAPE_LENGTH - 1)) / (ESCAPE_LENGTH - 1) * ESCAPE_LENGTH
}

pub fn encode(mut src: &[u8], dst: &mut Vec<u8>) {
    loop {
        let copy_len = cmp::min(ESCAPE_LENGTH - 1, src.len());
        dst.extend_from_slice(&src[0..copy_len]);
        src = &src[copy_len..];
        if src.is_empty() {
            let pad_size = ESCAPE_LENGTH - 1 - copy_len;
            if pad_size > 0 {
                dst.resize(dst.len() + pad_size, 0)
            }
            dst.push(copy_len as u8);
            break;
        }
        dst.push(ESCAPE_LENGTH as u8);
    }
}

pub fn decode(src: &mut &[u8], dst: &mut Vec<u8>) {
    loop {
        let extra = src[ESCAPE_LENGTH - 1];
        let len = cmp::min(ESCAPE_LENGTH - 1, extra as usize);
        dst.extend_from_slice(&src[..len]);
        *src = &src[ESCAPE_LENGTH..];
        if extra < ESCAPE_LENGTH as u8 {
            break;
        }
    }
}
