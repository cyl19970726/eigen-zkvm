const fs= require("fs");
const path = require("path");
const { polMulAxi } = require("pil-stark/src/polutils");
const version = require("../../package").version;
const JSONbig = require("json-bigint");

const argv = require("yargs")
    .version(version)
    .usage("node --zkin1 <in1.zkin.json> --zkin2 <in2.zkin.json>  --zkinout <out.zkin.json>")
    .argv;

async function run() {

    const zkin1File = typeof(argv.zkin1) === "string" ?  argv.zkin1.trim() : "zkin1.json";
    const zkin2File = typeof(argv.zkin2) === "string" ?  argv.zkin2.trim() : "zkin2.json";
    const zkinOutFile = typeof(argv.zkinout) === "string" ?  argv.zkinout : "zkinOut.json";

    const zkin1 = JSON.parse(await fs.promises.readFile(zkin1File, "utf8"));
    const zkin2 = JSON.parse(await fs.promises.readFile(zkin2File, "utf8"));

    const zkinOut = {};

    zkinOut.a_publics = zkin1.publics;
    zkinOut.a_rootC = zkin1.rootC;
    zkinOut.a_root1 = zkin1.root1;
    zkinOut.a_root2 = zkin1.root2;
    zkinOut.a_root3 = zkin1.root3;
    zkinOut.a_root4 = zkin1.root4;
    zkinOut.a_evals = zkin1.evals;
    zkinOut.a_s0_vals1 = zkin1.s0_vals1;
    zkinOut.a_s0_vals3 = zkin1.s0_vals3;
    zkinOut.a_s0_vals4 = zkin1.s0_vals4;
    zkinOut.a_s0_valsC = zkin1.s0_valsC;
    zkinOut.a_s0_siblings1 = zkin1.s0_siblings1;
    zkinOut.a_s0_siblings3 = zkin1.s0_siblings3;
    zkinOut.a_s0_siblings4 = zkin1.s0_siblings4;
    zkinOut.a_s0_siblingsC = zkin1.s0_siblingsC;
    zkinOut.a_s1_root = zkin1.s1_root;
    zkinOut.a_s2_root = zkin1.s2_root;
    zkinOut.a_s3_root = zkin1.s3_root;
    zkinOut.a_s4_root = zkin1.s4_root;
    zkinOut.a_s1_siblings = zkin1.s1_siblings;
    zkinOut.a_s2_siblings = zkin1.s2_siblings;
    zkinOut.a_s3_siblings = zkin1.s3_siblings;
    zkinOut.a_s4_siblings = zkin1.s4_siblings;
    zkinOut.a_s1_vals = zkin1.s1_vals;
    zkinOut.a_s2_vals = zkin1.s2_vals;
    zkinOut.a_s3_vals = zkin1.s3_vals;
    zkinOut.a_s4_vals = zkin1.s4_vals;
    zkinOut.a_finalPol = zkin1.finalPol;

    zkinOut.b_publics = zkin2.publics;
    zkinOut.b_rootC = zkin2.rootC;
    zkinOut.b_root1 = zkin2.root1;
    zkinOut.b_root2 = zkin2.root2;
    zkinOut.b_root3 = zkin2.root3;
    zkinOut.b_root4 = zkin2.root4;
    zkinOut.b_evals = zkin2.evals;
    zkinOut.b_s0_vals1 = zkin2.s0_vals1;
    zkinOut.b_s0_vals3 = zkin2.s0_vals3;
    zkinOut.b_s0_vals4 = zkin2.s0_vals4;
    zkinOut.b_s0_valsC = zkin2.s0_valsC;
    zkinOut.b_s0_siblings1 = zkin2.s0_siblings1;
    zkinOut.b_s0_siblings3 = zkin2.s0_siblings3;
    zkinOut.b_s0_siblings4 = zkin2.s0_siblings4;
    zkinOut.b_s0_siblingsC = zkin2.s0_siblingsC;
    zkinOut.b_s1_root = zkin2.s1_root;
    zkinOut.b_s2_root = zkin2.s2_root;
    zkinOut.b_s3_root = zkin2.s3_root;
    zkinOut.b_s4_root = zkin2.s4_root;
    zkinOut.b_s1_siblings = zkin2.s1_siblings;
    zkinOut.b_s2_siblings = zkin2.s2_siblings;
    zkinOut.b_s3_siblings = zkin2.s3_siblings;
    zkinOut.b_s4_siblings = zkin2.s4_siblings;
    zkinOut.b_s1_vals = zkin2.s1_vals;
    zkinOut.b_s2_vals = zkin2.s2_vals;
    zkinOut.b_s3_vals = zkin2.s3_vals;
    zkinOut.b_s4_vals = zkin2.s4_vals;
    zkinOut.b_finalPol = zkin2.finalPol;
    
    await fs.promises.writeFile(zkinOutFile, JSON.stringify(zkinOut, null, 1), "utf8");

    console.log("file Generated Correctly");

}

run().then(()=> {
    process.exit(0);
}, (err) => {
    console.log(err.message);
    console.log(err.stack);
    process.exit(1);
});
