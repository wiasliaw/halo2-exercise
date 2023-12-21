mod config;
mod chip;
mod circuit;

use circuit::FiboCircuit;

fn main() {
    use halo2_proofs::dev::MockProver;
    use halo2curves::pasta::Fp;

    let k = 6;
    let n = 9;
    // f(n+2) = f(n+1) + f(n)
    // f(0) = 1, f(1) = 1, f(9) = 55
    let out = Fp::from(55);

    let circuit = FiboCircuit {n};

    let public_input = vec![out];

    let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
    println!("success");
}
