mod config;
mod chip;
mod circuit;

use circuit::FiboCircuit;
use halo2_proofs::circuit::Value;

fn main() {
    use halo2_proofs::dev::MockProver;
    use halo2curves::pasta::Fp;

    let k = 4;
    let a = Value::known(Fp::from(1)); // f(0)
    let b = Value::known(Fp::from(1)); // f(1)
    let n = 8;
    let out = Fp::from(55);

    let circuit = FiboCircuit {
        a, b, n,
    };

    let public_input = vec![out];

    let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
    println!("success");
}
