pub fn list(seqs: &[u64]) -> Vec<u8> {
    let mut out = (seqs.len() as u32).to_le_bytes().to_vec();
    for seq in seqs {
        out.extend_from_slice(&seq.to_le_bytes());
    }
    out
}
