/*mod common;

use elusiv::poseidon::*;
use elusiv::groth16;
use ark_ff::*;
use common::*;

#[test]
fn test_valid_proof() {
    let proof = ProofString {
        ax: "15200472642106544087859624808573647436446459686589177220422407004547835364093",
        ay: "18563249006229852218279298661872929163955035535605917747249479039354347737308",
        az: "1",
        bx0: "20636553466803549451478361961314475483171634413642350348046906733449463808895",
        bx1: "3955337224043097728615186066317353350659966424133589619785214107405965410236",
        by0: "16669477906162214549333998971085624527095786690622350917799822973577201769757",
        by1: "10686129702127228201109048634021146893529704437134012687698468995076983569763",
        bz0: "1",
        bz1: "0",
        cx: "7825488021728597353611301562108479035418173715138578342437621330551207000521",
        cy: "17385834695111423269684287513728144523333186942287839669241715541894829818572",
        cz: "1",
    }.generate_proof();
    let inputs: Vec<Scalar> = vec![
        str_to_bigint("2213227377673335647524336988945236134992558051021751946022093019437366204428").into(),
        str_to_bigint("19810382324495243148399901112112255966654948517909945087601981242862578955507").into(),
    ];

    let own_result = groth16::verify_proof(inputs, proof);
    assert_eq!(own_result, true);
}

#[test]
fn test_invalid_proof() {
    let proof = ProofString {
        ax: "05200472642106544087859624808573647436446459686589177220422407004547835364093",
        ay: "18563249006229852218279298661872929163955035535605917747249479039354347737308",
        az: "1",
        bx0: "20636553466803549451478361961314475483171634413642350348046906733449463808895",
        bx1: "3955337224043097728615186066317353350659966424133589619785214107405965410236",
        by0: "16669477906162214549333998971085624527095786690622350917799822973577201769757",
        by1: "10686129702127228201109048634021146893529704437134012687698468995076983569763",
        bz0: "1",
        bz1: "0",
        cx: "7825488021728597353611301562108479035418173715138578342437621330551207000521",
        cy: "17385834695111423269684287513728144523333186942287839669241715541894829818572",
        cz: "1",
    }.generate_proof();
    let inputs: Vec<Scalar> = vec![
        str_to_bigint("2213227377673335647524336988945236134992558051021751946022093019437366204428").into(),
        str_to_bigint("19810382324495243148399901112112255966654948517909945087601981242862578955507").into(),
    ];

    let own_result = groth16::verify_proof(inputs, proof);
    assert_eq!(own_result, false);
}*/