use skrynia_program0::wire::{AppPrecond, Request};

pub fn req_cases() -> Vec<Request> {
    vec![
        Request::Register {
            operator_cap: vec![85; 32],
            admin_cap: vec![1; 32],
            code: vec![1, 2, 3],
        },
        Request::Set {
            admin_cap: vec![2; 32],
            expect: AppPrecond::Present(vec![9; 32]),
            code: vec![4, 5],
        },
        Request::Invoke {
            invoke_id: vec![7; 32],
            input: vec![8, 9],
        },
        Request::Lookup {
            invoke_id: vec![5; 32],
        },
        Request::Revoke {
            admin_cap: vec![6; 32],
            expect: AppPrecond::Absent,
        },
        Request::AppGet {
            read_cap: vec![3; 32],
            key: b"k".to_vec(),
        },
        Request::AppPut {
            write_cap: vec![4; 32],
            key: b"k".to_vec(),
            expect: AppPrecond::Absent,
            value: b"v".to_vec(),
        },
        Request::AppDelete {
            write_cap: vec![4; 32],
            key: b"k".to_vec(),
            expect: vec![8; 32],
        },
    ]
}
