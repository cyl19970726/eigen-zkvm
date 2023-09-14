# starkjs

PIL compiler and Circom transpiler. The stark prover is [starky](../starky).

## Run Example
### Arithmetization: Constraint Polynomial

```
export CIRCUIT=fib
npm run $CIRCUIT
```
will generate the PIL json, Commitment Polynomial file and Constant Polynomial file.

### Bottom Layer: FRI Proof

```
../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p /tmp/$CIRCUIT.pil.json \
    --o /tmp/$CIRCUIT.const \
    --m /tmp/$CIRCUIT.cm -c circuits/$CIRCUIT.verifier.circom --i circuits/$CIRCUIT.verifier.zkin.json
```

### Recursive Layer: FRI Proof

```
../target/release/eigen-zkit compile -p goldilocks -i circuits/$CIRCUIT.verifier.circom -l node_modules/pil-stark/circuits.gl --O2=full -o /tmp/

// Circom to Stark  
node src/compressor12/main_compressor12_setup.js \
    -r /tmp/$CIRCUIT.verifier.r1cs \
    -c /tmp/c12.const \
    -p /tmp/c12.pil \
    -e /tmp/c12.exec

node src/compressor12/main_compressor12_exec.js \
    -w /tmp/$CIRCUIT.verifier_js/$CIRCUIT.verifier.wasm  \
    -i circuits/$CIRCUIT.verifier.zkin.json  \
    -p /tmp/c12.pil  \
    -e /tmp/c12.exec \
    -m /tmp/c12.cm
../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.json \
    -p /tmp/c12.pil.json \
    --o /tmp/c12.const \
    --m /tmp/c12.cm -c circuits/c12a.verifier.circom --i circuits/c12a.verifier.zkin.json --norm_stage
```

### Top Layer: Snark proof
```
bash -x ../test/test_fibonacci_verifier.sh
```

### Snark proof aggregation

```
bash -x ../test/test_aggregation_verifier.sh
```

## Time used

CPU: 11th Gen Intel(R) Core(TM) i9-11900 @ 2.50GHz, 16core

MEM: 32G

| Step            | time(s) |
| ---             | ---     |
| Arithmetization | 0.021   |
| Bottom Layer    | 0.34    |
| Recursive Layer | 81.55   |
| Top Layer    | 267.2   |
