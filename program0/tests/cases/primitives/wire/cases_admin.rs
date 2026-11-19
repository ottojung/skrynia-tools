use skrynia_program0::wire::Request;
pub fn req_cases() -> Vec<Request> {
    vec![
        Request::StageCode {
            admin_cap: vec![1; 32],
            code: b"WASM".to_vec(),
        },
        Request::SealCode {
            admin_cap: vec![2; 32],
            hash: vec![3; 32],
        },
        Request::ActivateCode {
            admin_cap: vec![4; 32],
            expected_revision: 17,
            target: vec![5; 32],
        },
        Request::GetActive {
            invoke_id: vec![6; 32],
        },
        Request::ListVersions {
            invoke_id: vec![7; 32],
            after: None,
            limit: 256,
        },
        Request::ListVersions {
            invoke_id: vec![7; 32],
            after: Some(vec![8; 32]),
            limit: 13,
        },
        Request::ListPrograms {
            after: None,
            limit: 32,
        },
        Request::ListPrograms {
            after: Some(vec![9; 32]),
            limit: 1,
        },
    ]
}
