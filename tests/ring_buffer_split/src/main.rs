fn process_record(record: &mut [u64; 96], m_len: usize) -> u64 {
    assert!(m_len <= 32);
    let (acc, rest) = record.split_at_mut(m_len);
    let (base_cached, m_cached) = rest.split_at_mut(m_len);

    let mut checksum = 0_u64;
    for i in 0..m_len {
        acc[i] = acc[i].wrapping_add(base_cached[i]);
        m_cached[i] = m_cached[i].wrapping_add(acc[i]);
        checksum = checksum.wrapping_add(m_cached[i]);
    }
    checksum
}

fn main() {
    let mut record = [1_u64; 96];
    let checksum = process_record(&mut record, 32);
    println!("{checksum}");
}
