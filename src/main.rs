use std::arch::x86_64::*;

macro_rules! _mm_or_si128 {
    ($x:ident) => ($x);
    ($x:ident, $($y:ident),+) => (
        _mm_or_si128($x, _mm_or_si128!($($y),+))
    )
}


#[allow(non_snake_case)]
unsafe fn decode_16_bytes_simd(input: [u8; 16]) -> [u8; 16] {
    let input16: __m128i = std::mem::transmute(input);

    let vec_A = _mm_set1_epi8((b'A' -1) as i8);
    let vec_Z = _mm_set1_epi8((b'Z' +1) as i8);
    let vec_a = _mm_set1_epi8((b'a' -1) as i8);
    let vec_z = _mm_set1_epi8((b'z' +1) as i8);
    let vec_0 = _mm_set1_epi8((b'0' -1) as i8);
    let vec_9 = _mm_set1_epi8((b'9' +1) as i8);

    // A-Z => 0-25
    let greater_than_A = _mm_cmpgt_epi8(input16, vec_A);
    let less_than_Z = _mm_cmplt_epi8(input16, vec_Z);
    let between_A_Z = _mm_and_si128(greater_than_A, less_than_Z);
    let between_A_Z_mask = _mm_and_si128(_mm_set1_epi8(-(b'A' as i8)), between_A_Z);

    // a-z => 26-51
    let greater_than_a = _mm_cmpgt_epi8(input16, vec_a);
    let less_than_z = _mm_cmplt_epi8(input16, vec_z);
    let between_a_z = _mm_and_si128(greater_than_a, less_than_z);
    let between_a_z_mask = _mm_and_si128(_mm_set1_epi8(-(b'a' as i8) + 26), between_a_z);

    // 0-9 => 52-61
    let greater_than_0 = _mm_cmpgt_epi8(input16, vec_0);
    let less_than_9 = _mm_cmplt_epi8(input16, vec_9);
    let between_0_9 = _mm_and_si128(greater_than_0, less_than_9);
    let between_0_9_mask = _mm_and_si128(_mm_set1_epi8(-(b'0' as i8) + 52), between_0_9);

    // + => 62
    let vec_plus = _mm_set1_epi8(b'+' as i8);
    let equals_plus = _mm_cmpeq_epi8(input16, vec_plus);
    let equals_plus_mask = _mm_and_si128(_mm_set1_epi8(-(b'+' as i8) + 62), equals_plus);

    // / => 63
    let vec_slash = _mm_set1_epi8(b'/' as i8);
    let equals_slash = _mm_cmpeq_epi8(input16, vec_slash);
    let equals_slash_mask = _mm_and_si128(_mm_set1_epi8(-(b'/' as i8) + 62), equals_slash);

    let combined = _mm_or_si128!(between_A_Z_mask, between_a_z_mask, between_0_9_mask, equals_plus_mask, equals_slash_mask);
    let res = _mm_add_epi8(input16, combined);

    std::mem::transmute(res)
}


#[allow(non_snake_case)]
unsafe fn encode_16_bytes_simd(input: [u8; 16]) -> [u8; 16] {
    let input16: __m128i = std::mem::transmute(input);
    
    // A-Z => 0-25
    let vec_A = _mm_set1_epi8(-1);
    let vec_Z = _mm_set1_epi8(26);
    let greater_than_A = _mm_cmpgt_epi8(input16, vec_A);
    let less_than_Z = _mm_cmplt_epi8(input16, vec_Z);
    let between_A_Z = _mm_and_si128(greater_than_A, less_than_Z);
    let between_A_Z_mask = _mm_and_si128(_mm_set1_epi8(b'A' as i8), between_A_Z);

    // a-z => 26-51
    let vec_a = _mm_set1_epi8(25);
    let vec_z = _mm_set1_epi8(52);
    let greater_than_a = _mm_cmpgt_epi8(input16, vec_a);
    let less_than_z = _mm_cmplt_epi8(input16, vec_z);
    let between_a_z = _mm_and_si128(greater_than_a, less_than_z);
    let between_a_z_mask = _mm_and_si128(_mm_set1_epi8(b'a' as i8 - 26), between_a_z);

    // 0-9 => 52-61
    let vec_0 = _mm_set1_epi8(51);
    let vec_9 = _mm_set1_epi8(62);
    let greater_than_0 = _mm_cmpgt_epi8(input16, vec_0);
    let less_than_9 = _mm_cmplt_epi8(input16, vec_9);
    let between_0_9 = _mm_and_si128(greater_than_0, less_than_9);
    let between_0_9_mask = _mm_and_si128(_mm_set1_epi8(b'0' as i8 - 52), between_0_9);
    
    let vec_plus = _mm_set1_epi8(62);
    let equals_plus = _mm_cmpeq_epi8(input16, vec_plus);
    let equals_plus_mask = _mm_and_si128(_mm_set1_epi8(b'+' as i8), equals_plus);

    let vec_slash = _mm_set1_epi8(63);
    let equals_slash = _mm_cmpeq_epi8(input16, vec_slash);
    let equals_slash_mask = _mm_and_si128(_mm_set1_epi8(b'/' as i8), equals_slash);

    let combined = _mm_or_si128!(between_A_Z_mask, between_a_z_mask, between_0_9_mask, equals_plus_mask, equals_slash_mask);
    let res = _mm_add_epi8(input16, combined); 

    std::mem::transmute(res)
}


struct BitStream<'a> {
    data: &'a [u8],
}

impl<'a> BitStream<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn encode_into_iter(self) -> EncodeIntoIter<'a> {
        EncodeIntoIter {
            data: self.data,
            index: 0,
            bit_index: 0,
        }
    }

    fn decode_into_iter(self) -> DecodeIntoIter<'a> {
        DecodeIntoIter {
            data: self.data,
            index: 1,
            left_from_last: 6,
        }
    }
}

struct EncodeIntoIter<'a> {
    data: &'a [u8],
    index: usize,
    bit_index: isize,
}

impl<'a> Iterator for EncodeIntoIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let bits_from_last = self.index as isize * 8 - self.bit_index;

        let current = if self.index < self.data.len() {
            self.data[self.index]
        } else if self.index == self.data.len() && self.bit_index < self.index as isize * 8 {
            self.data[self.index-1] >> (8 - bits_from_last)
        } else {
            return None
        };

        self.bit_index += 6;

        if self.bit_index % 8 != 2 {
            self.index += 1;
        }

        if bits_from_last == -2 {
            return Some(current & 0b0011_1111)
        }
        if bits_from_last == 0 {
            return Some(current >> 2)
        }
        if bits_from_last == 2 {
            let last = self.data[self.index-2] << 4 & 0b0011_0000;
            let current = current >> 4;
            return Some(last + current)
        }
        if bits_from_last == 4 {
            let last = self.data[self.index-1] << 2 & 0b0011_1100;
            let current = current >> 6;
            return Some(last + current)
        }

        None
    }
}

struct DecodeIntoIter<'a> {
    data: &'a [u8],
    index: usize,
    left_from_last: u8,
}

impl<'a> Iterator for DecodeIntoIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.left_from_last == 0 {
            self.index += 1;
            self.left_from_last = 6;
        }
        let current = self.data.get(self.index)?;
        let last = self.data[self.index-1];
        let shift = match self.left_from_last {
            2 => 6,
            4 => 4,
            6 => 2,
            _ => panic!("Invalid state"),
        };

        let last = last << shift;
        let current = current >> (self.left_from_last - 2);

        self.index += 1;
        self.left_from_last = 6 - (8 - self.left_from_last);
        Some(last + current)
    }
}

fn decode(data: &[u8]) -> String {
    let mut b64data = Vec::with_capacity(data.len());
    for chunk in data.chunks(16) {
        let mut byte_array = [0u8; 16];
        byte_array[..chunk.len()].copy_from_slice(chunk);

        let decoded = unsafe { decode_16_bytes_simd(byte_array) };
        b64data.extend_from_slice(&decoded[..chunk.len()]);
    }

    let stream = BitStream::new(&b64data).decode_into_iter();
    let mut output = stream.collect::<Vec<_>>();

    let mut index = output.len() - 1;
    while index > 0 && output[index] == b'=' {
        output.pop();
        index -= 1;
    }

    unsafe { String::from_utf8_unchecked(output) }
}

fn encode(data: &[u8]) -> String {
    let mut output = Vec::with_capacity(data.len());
    let stream = BitStream::new(data).encode_into_iter();

    for chunk in stream.collect::<Vec<_>>().chunks(16) {
        let mut byte_array = [0u8; 16];
        byte_array[..chunk.len()].copy_from_slice(chunk);

        let encoded = unsafe { encode_16_bytes_simd(byte_array) };
        output.extend_from_slice(&encoded[..chunk.len()]);
    }

    let pad = match data.len() % 3 {
        1 => 2,
        2 => 1,
        _ => 0,
    };
    output.extend(std::iter::repeat(b'=').take(pad));

    String::from_utf8(output).unwrap()
}


fn main() {
    let mut args = std::env::args();
    let action = args.nth(1).expect("user should have provided action");
    let value = args.next().expect("user should have provided data");

    let ret = if action == "-d" || action == "--decode" {
        decode(value.as_bytes())
    } else if action == "-e" || action == "--encode" {
        encode(value.as_bytes())
    } else {
        panic!("Invalid action");
    };

    println!("{ret}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_iterator() {
        let text = "ManMan";
        let stream = BitStream::new(text.as_bytes()).encode_into_iter();

        let encoded = stream.collect::<Vec<_>>();
        assert_eq!(encoded, vec![19, 22, 5, 46, 19, 22, 5, 46]);
    }

    #[test]
    fn test_encode() {
        let data = "encode me senpai uwu :3";
        let expected = "ZW5jb2RlIG1lIHNlbnBhaSB1d3UgOjM=";

        let encoded = encode(data.as_bytes());
        assert_eq!(expected, encoded)
    }

    #[test]
    fn test_padding() {
        let test1 = "light work.";	
        let test2 = "light work";  
        let test3 = "light wor";  
        let test4 = "light wo";  
        let test5 = "light w";  

        let ans1 = "bGlnaHQgd29yay4="; 	
        let ans2 = "bGlnaHQgd29yaw=="; 	    
        let ans3 = "bGlnaHQgd29y"; 	 
        let ans4 = "bGlnaHQgd28="; 	
        let ans5 = "bGlnaHQgdw==";

        assert_eq!(encode(test1.as_bytes()).as_bytes(), ans1.as_bytes());
        assert_eq!(encode(test2.as_bytes()).as_bytes(), ans2.as_bytes());
        assert_eq!(encode(test3.as_bytes()).as_bytes(), ans3.as_bytes());
        assert_eq!(encode(test4.as_bytes()).as_bytes(), ans4.as_bytes());
        assert_eq!(encode(test5.as_bytes()).as_bytes(), ans5.as_bytes());
    }

    #[test]
    fn test_decoding() {
        let data = "TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu";
        let ans = "Many hands make light work.";
        assert_eq!(decode(data.as_bytes()), ans);

        let data = "TGluZTEKICAgIExpbmUyCiAgICAgICAgTGluZTM=";
        let ans = 
"Line1
    Line2
        Line3";
        assert_eq!(decode(data.as_bytes()), ans);
    }
}
