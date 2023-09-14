#![allow(non_snake_case, dead_code)]
use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MG, MIN_OPS_PER_THREAD, SHIFT};
use crate::errors::Result;
use crate::f3g::F3G;
use crate::fft::FFT;
use crate::fft_p::{fft, ifft, interpolate};
use crate::fri::FRIProof;
use crate::fri::FRI;
use crate::helper::pretty_print_array;
use crate::interpreter::compile_code;
use crate::polsarray::PolsArray;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{Polynom, Segment};
use crate::traits::{MTNodeType, MerkleTree, Transcript};
use crate::types::{StarkStruct, PIL};
use plonky::field_gl::Fr as FGL;
use plonky::Field;
use rayon::prelude::*;
use std::collections::HashMap;

pub struct StarkContext {
    pub nbits: usize,
    pub nbits_ext: usize,
    pub N: usize,
    pub Next: usize,
    pub challenge: Vec<F3G>,
    pub tmp: Vec<F3G>,
    pub cm1_n: Vec<F3G>,
    pub cm2_n: Vec<F3G>,
    pub cm3_n: Vec<F3G>,
    pub cm4_n: Vec<F3G>,
    pub tmpexp_n: Vec<F3G>,
    pub cm1_2ns: Vec<F3G>,
    pub cm2_2ns: Vec<F3G>,
    pub cm3_2ns: Vec<F3G>,
    pub cm4_2ns: Vec<F3G>,
    pub q_2ns: Vec<F3G>,
    pub f_2ns: Vec<F3G>,
    pub x_n: Vec<F3G>,
    pub x_2ns: Vec<F3G>,
    pub Zi: Box<dyn Fn(usize) -> F3G>,
    pub const_n: Vec<F3G>,
    pub const_2ns: Vec<F3G>,
    pub publics: Vec<F3G>,
    pub xDivXSubXi: Vec<FGL>,
    pub xDivXSubWXi: Vec<FGL>,
    pub evals: Vec<F3G>,

    pub exps_n: Vec<F3G>,
    pub exps_2ns: Vec<F3G>,

    pub Z: F3G,
    pub Zp: F3G,
    pub tree1: Vec<FGL>,
    pub tree2: Vec<FGL>,
    pub tree3: Vec<FGL>,
    pub tree4: Vec<FGL>,
    pub consts: Vec<FGL>,
}

impl std::fmt::Debug for StarkContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "n {}", self.N)?;
        writeln!(f, "nBits {}", self.nbits)?;
        writeln!(f, "nBitsExt {}", self.nbits_ext)?;
        writeln!(f, "evals {}", pretty_print_array(&self.evals))?;
        writeln!(f, "publics {}", pretty_print_array(&self.publics))?;
        writeln!(f, "challenge {}", pretty_print_array(&self.challenge))?;
        writeln!(f, r#"cm1_n {}"#, pretty_print_array(&self.cm1_n))?;
        writeln!(f, "cm2_n {}", pretty_print_array(&self.cm2_n))?;
        writeln!(f, "cm3_n {}", pretty_print_array(&self.cm3_n))?;
        writeln!(f, "cm4_n {}", pretty_print_array(&self.cm4_n))?;
        writeln!(f, "cm1_2ns {}", pretty_print_array(&self.cm1_2ns))?;
        writeln!(f, "cm2_2ns {}", pretty_print_array(&self.cm2_2ns))?;
        writeln!(f, "cm3_2ns {}", pretty_print_array(&self.cm3_2ns))?;
        writeln!(f, "cm4_2ns {}", pretty_print_array(&self.cm4_2ns))?;
        writeln!(f, "const_n {}", pretty_print_array(&self.const_n))?;
        writeln!(f, "const_2ns {}", pretty_print_array(&self.const_2ns))?;
        writeln!(f, "x_n {}", pretty_print_array(&self.x_n))?;
        writeln!(f, "x_2ns {}", pretty_print_array(&self.x_2ns))?;
        writeln!(f, "xDivXSubXi {}", pretty_print_array(&self.xDivXSubXi))?;
        writeln!(f, "xDivXSubWXi {}", pretty_print_array(&self.xDivXSubWXi))?;
        writeln!(f, "q_2ns {}", pretty_print_array(&self.q_2ns))?;
        writeln!(f, "f_2ns {}", pretty_print_array(&self.f_2ns))?;
        writeln!(f, "tmp {}", pretty_print_array(&self.tmp))?;

        Ok(())
    }
}

unsafe impl Send for StarkContext {}
unsafe impl Sync for StarkContext {}

impl Default for StarkContext {
    fn default() -> Self {
        StarkContext {
            nbits: 0,
            nbits_ext: 0,
            N: 0,
            Next: 0,
            challenge: vec![F3G::ZERO; 8],
            tmp: Vec::new(),
            cm1_n: Vec::new(),
            cm2_n: Vec::new(),
            cm3_n: Vec::new(),
            cm4_n: Vec::new(),
            tmpexp_n: Vec::new(),
            cm1_2ns: Vec::new(),
            cm2_2ns: Vec::new(),
            cm3_2ns: Vec::new(),
            cm4_2ns: Vec::new(),
            q_2ns: Vec::new(),
            f_2ns: Vec::new(),
            x_n: Vec::new(),
            x_2ns: Vec::new(),
            Zi: Box::new(|_: usize| F3G::ZERO),
            const_n: Vec::new(),
            const_2ns: Vec::new(),
            publics: Vec::new(),
            xDivXSubXi: Vec::new(),
            xDivXSubWXi: Vec::new(),
            evals: Vec::new(),
            exps_n: Vec::new(),
            exps_2ns: Vec::new(),
            Z: F3G::ZERO,
            Zp: F3G::ZERO,
            tree1: Vec::new(),
            tree2: Vec::new(),
            tree3: Vec::new(),
            tree4: Vec::new(),
            consts: Vec::new(),
        }
    }
}

impl StarkContext {
    pub fn get_mut_base(&mut self, section: &str) -> &mut Vec<FGL> {
        match section {
            "xDivXSubXi" => &mut self.xDivXSubXi,
            "xDivXSubWXi" => &mut self.xDivXSubWXi,
            _ => panic!("invalid symbol {:?}", section),
        }
    }
    pub fn get_mut(&mut self, section: &str) -> &mut Vec<F3G> {
        match section {
            "tmp" => &mut self.tmp,
            "cm1_n" => &mut self.cm1_n,
            "cm1_2ns" => &mut self.cm1_2ns,
            "cm2_n" => &mut self.cm2_n,
            "cm2_2ns" => &mut self.cm2_2ns,
            "cm3_n" => &mut self.cm3_n,
            "cm4_n" => &mut self.cm4_n,
            "cm3_2ns" => &mut self.cm3_2ns,
            "cm4_2ns" => &mut self.cm4_2ns,
            "q_2ns" => &mut self.q_2ns,
            "f_2ns" => &mut self.f_2ns,
            "exps_n" => &mut self.exps_n,
            "exps_2ns" => &mut self.exps_2ns,
            "const_n" => &mut self.const_n,
            "const_2ns" => &mut self.const_2ns,
            "evals" => &mut self.evals,
            "publics" => &mut self.publics,
            "challenge" => &mut self.challenge,
            "tmpexp_n" => &mut self.tmpexp_n,
            "x_n" => &mut self.x_n,
            "x_2ns" => &mut self.x_2ns,
            _ => {
                panic!("invalid symbol {:?}", section);
            }
        }
    }
}

pub struct StarkProof<M: MerkleTree> {
    pub root1: M::MTNode,
    pub root2: M::MTNode,
    pub root3: M::MTNode,
    pub root4: M::MTNode,
    pub fri_proof: FRIProof<M>,
    pub evals: Vec<F3G>,
    pub publics: Vec<F3G>,
    pub rootC: Option<M::MTNode>,
    pub stark_struct: StarkStruct,
}

impl<'a, M: MerkleTree> StarkProof<M> {
    pub fn stark_gen<T: Transcript>(
        cm_pols: &PolsArray,
        const_pols: &PolsArray,
        const_tree: &M,
        starkinfo: &'a StarkInfo,
        program: &Program,
        _pil: &PIL,
        stark_struct: &StarkStruct,
    ) -> Result<StarkProof<M>> {
        let mut ctx = StarkContext::default();
        //log::debug!("starkinfo: {}", starkinfo);
        //log::debug!("program: {}", program);

        let mut fftobj = FFT::new();
        ctx.nbits = stark_struct.nBits;
        ctx.nbits_ext = stark_struct.nBitsExt;
        ctx.N = 1 << stark_struct.nBits;
        ctx.Next = 1 << stark_struct.nBitsExt;
        assert_eq!(1 << ctx.nbits, ctx.N, "N must be a power of 2");

        let mut n_cm = starkinfo.n_cm1;

        ctx.cm1_n = cm_pols.write_buff();
        ctx.cm2_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm2_n) * ctx.N];
        ctx.cm3_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm3_n) * ctx.N];
        ctx.tmpexp_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.tmpexp_n) * ctx.N];

        ctx.cm1_2ns = vec![F3G::ZERO; starkinfo.map_sectionsN.cm1_n * ctx.Next];
        ctx.cm2_2ns = vec![F3G::ZERO; starkinfo.map_sectionsN.cm2_n * ctx.Next];
        ctx.cm3_2ns = vec![F3G::ZERO; starkinfo.map_sectionsN.cm3_n * ctx.Next];
        ctx.cm4_2ns = vec![F3G::ZERO; starkinfo.map_sectionsN.cm4_n * ctx.Next];
        ctx.const_2ns = vec![F3G::ZERO; const_tree.element_size()];

        ctx.q_2ns = vec![F3G::ZERO; starkinfo.q_dim * ctx.Next];
        ctx.f_2ns = vec![F3G::ZERO; 3 * ctx.Next];

        ctx.x_n = vec![F3G::ZERO; ctx.N];
        let mut xx = F3G::ONE;
        let w_nbits = MG.0[ctx.nbits];
        for i in 0..ctx.N {
            ctx.x_n[i] = xx;
            xx = xx * w_nbits;
        }

        let extend_bits = ctx.nbits_ext - ctx.nbits;
        ctx.x_2ns = vec![F3G::ZERO; ctx.N << extend_bits];
        let mut xx = SHIFT.clone();
        for i in 0..(ctx.N << extend_bits) {
            ctx.x_2ns[i] = xx;
            xx = xx * MG.0[ctx.nbits_ext];
        }

        ctx.Zi = build_Zh_Inv(ctx.nbits, extend_bits, 0);

        ctx.const_n = const_pols.write_buff();
        const_tree.to_f3g(&mut ctx.const_2ns);

        ctx.publics = vec![F3G::ZERO; starkinfo.publics.len()];
        for (i, pe) in starkinfo.publics.iter().enumerate() {
            if pe.polType.as_str() == "cmP" {
                ctx.publics[i] = ctx.cm1_n[pe.idx * starkinfo.map_sectionsN.cm1_n + pe.polId];
            } else if pe.polType.as_str() == "imP" {
                ctx.publics[i] = Self::calculate_exp_at_point(
                    &mut ctx,
                    starkinfo,
                    &program.publics_code[i],
                    pe.idx,
                );
            } else {
                panic!("Invalid public type {}", pe.polType);
            }
        }

        let mut transcript = T::new();
        for i in 0..starkinfo.publics.len() {
            let b = ctx.publics[i]
                .as_elements()
                .iter()
                .map(|e| vec![e.clone()])
                .collect::<Vec<Vec<FGL>>>();
            transcript.put(&b[..])?;
        }

        log::info!("Merkelizing 1....");
        let tree1 = extend_and_merkelize::<M>(&mut ctx, starkinfo, "cm1_n")?;
        tree1.to_f3g(&mut ctx.cm1_2ns);

        log::info!(
            "tree1 root: {}",
            //crate::helper::fr_to_biguint(&tree1.root().into())
            tree1.root(),
        );
        transcript.put(&[tree1.root().as_elements().to_vec()])?;
        // 2.- Caluculate plookups h1 and h2
        ctx.challenge[0] = transcript.get_field(); //u
        ctx.challenge[1] = transcript.get_field(); //defVal

        log::debug!("challenge[0] {}", ctx.challenge[0]);
        log::debug!("challenge[1] {}", ctx.challenge[1]);

        calculate_exps_parallel(&mut ctx, starkinfo, &program.step2prev, "n", "step2prev");

        for pu in starkinfo.pu_ctx.iter() {
            let f_pol = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pu.f_exp_id]);
            let t_pol = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pu.t_exp_id]);
            let (h1, h2) = calculate_H1H2(f_pol, t_pol);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], h1);
            n_cm += 1;
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], h2);
            n_cm += 1;
        }

        log::info!("Merkelizing 2....");
        let tree2 = extend_and_merkelize::<M>(&mut ctx, starkinfo, "cm2_n")?;
        tree2.to_f3g(&mut ctx.cm2_2ns);
        transcript.put(&[tree2.root().as_elements().to_vec()])?;
        log::info!(
            "tree2 root: {}",
            // crate::helper::fr_to_biguint(&tree2.root().into())
            tree2.root(),
        );

        // 3.- Compute Z polynomials
        ctx.challenge[2] = transcript.get_field(); // gamma
        ctx.challenge[3] = transcript.get_field(); // betta
        log::debug!("challenge[2] {}", ctx.challenge[2]);
        log::debug!("challenge[3] {}", ctx.challenge[3]);

        calculate_exps_parallel(&mut ctx, starkinfo, &program.step3prev, "n", "step3prev");

        for (i, pu) in starkinfo.pu_ctx.iter().enumerate() {
            log::info!("Calculating z for plookup {}", i);
            let p_num = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pu.num_id]);
            let p_den = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pu.den_id]);
            let z = calculate_Z(p_num, p_den);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }

        for (i, pe) in starkinfo.pe_ctx.iter().enumerate() {
            log::info!("Calculating z for permutation {}", i);
            let p_num = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pe.num_id]);
            let p_den = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pe.den_id]);
            let z = calculate_Z(p_num, p_den);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }
        for (i, ci) in starkinfo.ci_ctx.iter().enumerate() {
            log::info!("Calculating z for connection {}", i);
            let p_num = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&ci.num_id]);
            let p_den = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&ci.den_id]);
            let z = calculate_Z(p_num, p_den);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }

        calculate_exps_parallel(&mut ctx, starkinfo, &program.step3, "n", "step3");

        log::info!("Merkelizing 3....");

        let tree3 = extend_and_merkelize::<M>(&mut ctx, starkinfo, "cm3_n")?;
        tree3.to_f3g(&mut ctx.cm3_2ns);
        transcript.put(&[tree3.root().as_elements().to_vec()])?;

        log::info!(
            "tree3 root: {}",
            // crate::helper::fr_to_biguint(&tree3.root().into())
            tree3.root(),
        );

        // 4. Compute C Polynomial
        ctx.challenge[4] = transcript.get_field(); // vc

        calculate_exps_parallel(&mut ctx, starkinfo, &program.step42ns, "2ns", "step4");

        let mut qq1 = vec![F3G::ZERO; ctx.q_2ns.len()];
        let mut qq2 = vec![F3G::ZERO; starkinfo.q_dim * ctx.Next * starkinfo.q_deg];
        ifft(&ctx.q_2ns, starkinfo.q_dim, ctx.nbits_ext, &mut qq1);

        let mut cur_s = F3G::ONE;
        let shift_in = (F3G::inv(SHIFT.clone())).exp(ctx.N);
        for p in 0..starkinfo.q_deg {
            for i in 0..ctx.N {
                for k in 0..starkinfo.q_dim {
                    qq2[i * starkinfo.q_dim * starkinfo.q_deg + starkinfo.q_dim * p + k] =
                        qq1[p * ctx.N * starkinfo.q_dim + i * starkinfo.q_dim + k] * cur_s;
                }
            }
            cur_s = cur_s * shift_in;
        }

        if starkinfo.q_deg > 0 {
            fft(
                &qq2,
                starkinfo.q_dim * starkinfo.q_deg,
                ctx.nbits_ext,
                &mut ctx.cm4_2ns,
            );
        }

        log::info!("Merkelizing 4....");
        let tree4 = merkelize::<M>(&mut ctx, starkinfo, "cm4_2ns").unwrap();
        log::info!(
            "tree4 root: {}",
            // crate::helper::fr_to_biguint(&tree4.root().into())
            tree4.root(),
        );
        transcript.put(&[tree4.root().as_elements().to_vec()])?;

        //if ctx.cm4_2ns.len() > 0 {
        //    log::info!("tree4[0] {}", ctx.cm4_2ns[0]);
        //}

        ///////////
        // 5. Compute FRI Polynomial
        ///////////
        ctx.challenge[7] = transcript.get_field(); // xi

        let mut LEv = vec![F3G::ZERO; ctx.N];
        let mut LpEv = vec![F3G::ZERO; ctx.N];
        LEv[0] = F3G::from(FGL::from(1u64));
        LpEv[0] = F3G::from(FGL::from(1u64));

        let xis = ctx.challenge[7] / SHIFT.clone();
        let wxis = (ctx.challenge[7] * MG.0[ctx.nbits]) / SHIFT.clone();

        for i in 1..ctx.N {
            LEv[i] = LEv[i - 1] * xis;
            LpEv[i] = LpEv[i - 1] * wxis;
        }

        let LEv = fftobj.ifft(&LEv);
        let LpEv = fftobj.ifft(&LpEv);

        ctx.evals = vec![F3G::ZERO; starkinfo.ev_map.len()];
        let N = ctx.N;
        for (i, ev) in starkinfo.ev_map.iter().enumerate() {
            let p = match ev.type_.as_str() {
                "const" => Polynom {
                    buffer: &mut ctx.const_2ns,
                    deg: 1 << ctx.nbits_ext,
                    offset: ev.id,
                    size: starkinfo.n_constants,
                    dim: 1,
                },
                "cm" => get_pol_ref(&mut ctx, starkinfo, starkinfo.cm_2ns[ev.id]),
                _ => {
                    panic!("Invalid ev type: {}", ev.type_);
                }
            };
            let l = if ev.prime { &LpEv } else { &LEv };
            log::debug!("calculate acc: N={}", N);
            let acc = (0..N)
                .into_par_iter()
                .map(|k| {
                    let v = match p.dim {
                        1 => p.buffer[(k << extend_bits) * (p.size) + (p.offset)],
                        _ => F3G::new(
                            p.buffer[p.offset + (k << extend_bits) * (p.size)].to_be(),
                            p.buffer[p.offset + (k << extend_bits) * (p.size) + 1].to_be(),
                            p.buffer[p.offset + (k << extend_bits) * (p.size) + 2].to_be(),
                        ),
                    };
                    v * l[k]
                })
                .reduce(|| F3G::ZERO, |a, b| a + b);
            ctx.evals[i] = acc;
        }

        for i in 0..ctx.evals.len() {
            let b = ctx.evals[i]
                .as_elements()
                .iter()
                .map(|e| vec![e.clone()])
                .collect::<Vec<Vec<FGL>>>();
            transcript.put(&b)?;
        }

        ctx.challenge[5] = transcript.get_field(); // v1
        ctx.challenge[6] = transcript.get_field(); // v2
        log::debug!("ctx.challenge[5] {}", ctx.challenge[5]);
        log::debug!("ctx.challenge[6] {}", ctx.challenge[6]);
        log::debug!("ctx.challenge[7] {}", ctx.challenge[7]);

        // Calculate xDivXSubXi, xDivXSubWXi
        let xi = ctx.challenge[7];
        let wxi = ctx.challenge[7] * MG.0[ctx.nbits];

        let extend_size = N << extend_bits;

        ctx.xDivXSubXi = vec![FGL::ZERO; extend_size * 3];
        ctx.xDivXSubWXi = vec![FGL::ZERO; extend_size * 3];
        let mut tmp_den = vec![F3G::ZERO; extend_size];
        let mut tmp_denw = vec![F3G::ZERO; extend_size];

        let mut x_buff = vec![F3G::ZERO; extend_size];

        x_buff.par_iter_mut().enumerate().for_each(|(k, xb)| {
            *xb = SHIFT.clone() * (MG.0[ctx.nbits + extend_bits].exp(k));
        });

        tmp_den
            .par_iter_mut()
            .zip_eq(tmp_denw.par_iter_mut())
            .enumerate()
            .for_each(|(k, (td, tdw))| {
                *td = x_buff[k] - xi;
                *tdw = x_buff[k] - wxi;
            });

        tmp_den = F3G::batch_inverse(&tmp_den);
        tmp_denw = F3G::batch_inverse(&tmp_denw);
        ctx.xDivXSubXi
            .par_chunks_mut(3)
            .zip_eq(ctx.xDivXSubWXi.par_chunks_mut(3))
            .enumerate()
            .for_each(|(k, (xxx, xxwx))| {
                let v = (tmp_den[k] * x_buff[k]).as_elements();
                xxx[0] = v[0];
                xxx[1] = v[1];
                xxx[2] = v[2];

                let vw = (tmp_denw[k] * x_buff[k]).as_elements();
                xxwx[0] = vw[0];
                xxwx[1] = vw[1];
                xxwx[2] = vw[2];
            });
        calculate_exps_parallel(&mut ctx, starkinfo, &program.step52ns, "2ns", "step5");

        let mut fri_pol = vec![F3G::ZERO; N << extend_bits];
        fri_pol.par_iter_mut().enumerate().for_each(|(i, o)| {
            *o = F3G::new(
                ctx.f_2ns[i * 3].to_be(),
                ctx.f_2ns[i * 3 + 1].to_be(),
                ctx.f_2ns[i * 3 + 2].to_be(),
            );
        });

        let query_pol = |idx: usize| -> Vec<(Vec<FGL>, Vec<Vec<M::BaseField>>)> {
            vec![
                tree1.get_group_proof(idx).unwrap(),
                tree2.get_group_proof(idx).unwrap(),
                tree3.get_group_proof(idx).unwrap(),
                tree4.get_group_proof(idx).unwrap(),
                const_tree.get_group_proof(idx).unwrap(),
            ]
        };
        let mut fri = FRI::new(stark_struct);
        let friProof = fri.prove::<M, T>(&mut transcript, &fri_pol, query_pol)?;

        Ok(StarkProof {
            rootC: Some(const_tree.root()),
            root1: tree1.root(),
            root2: tree2.root(),
            root3: tree3.root(),
            root4: tree4.root(),
            fri_proof: friProof,
            evals: ctx.evals.clone(),
            publics: ctx.publics.clone(),
            stark_struct: stark_struct.clone(),
        })
    }

    pub fn calculate_exp_at_point(
        ctx: &mut StarkContext,
        starkinfo: &StarkInfo,
        seg: &Segment,
        idx: usize,
    ) -> F3G {
        ctx.tmp = vec![F3G::ZERO; seg.tmp_used];
        let t = compile_code(ctx, starkinfo, &seg.first, "n", true);
        log::debug!("calculate_exp_at_point compile_code ctx.first:\n{}", t);

        let res = t.eval(ctx, idx); // just let public codegen run multiple times
                                    //log::debug!("{} = {} @ {}", res, ctx.cm1_n[1 + 2 * idx], idx);
        res
    }
}

pub fn build_Zh_Inv(
    nBits: usize,
    extend_bits: usize,
    offset: usize,
) -> Box<dyn Fn(usize) -> F3G + 'static> {
    let mut w = F3G::ONE;
    let mut sn = SHIFT.clone();
    for _i in 0..nBits {
        sn = sn * sn;
    }
    let mut ZHInv = vec![F3G::ZERO; 1 << extend_bits];
    for i in 0..(1 << extend_bits) {
        ZHInv[i] = F3G::inv(sn * w - F3G::ONE);
        w = w * MG.0[extend_bits];
    }
    Box::new(move |i: usize| ZHInv[(i + offset) % ZHInv.len()].clone())
}

fn set_pol(ctx: &mut StarkContext, starkinfo: &StarkInfo, id_pol: &usize, pol: Vec<F3G>) {
    let id_pol = *id_pol;
    let p = get_pol_ref(ctx, starkinfo, id_pol);
    if p.dim == 1 {
        for i in 0..p.deg {
            p.buffer[p.offset + i * p.size] = pol[i];
        }
    } else if p.dim == 3 {
        for i in 0..p.deg {
            let elems = pol[i].as_elements();
            if elems.len() > 1 {
                p.buffer[p.offset + i * p.size] = elems[0].into();
                p.buffer[p.offset + i * p.size + 1] = elems[1].into();
                p.buffer[p.offset + i * p.size + 2] = elems[2].into();
            } else {
                p.buffer[p.offset + i * p.size] = elems[0].into();
                p.buffer[p.offset + i * p.size + 1] = F3G::ZERO;
                p.buffer[p.offset + i * p.size + 2] = F3G::ZERO;
            }
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
}

fn calculate_H1H2(f: Vec<F3G>, t: Vec<F3G>) -> (Vec<F3G>, Vec<F3G>) {
    let mut idx_t: HashMap<F3G, usize> = HashMap::new();
    let mut s: Vec<(F3G, usize)> = vec![];

    for (i, e) in t.iter().enumerate() {
        idx_t.insert(*e, i);
        s.push((*e, i));
    }

    for (_i, e) in f.iter().enumerate() {
        let idx = idx_t.get(e);
        if idx.is_none() {
            panic!("Number not included: {:?}", e);
        }
        s.push((e.clone(), *idx.unwrap()));
    }

    s.sort_by(|a, b| a.1.cmp(&b.1));

    let mut h1 = vec![F3G::ZERO; f.len()];
    let mut h2 = vec![F3G::ZERO; f.len()];
    for i in 0..f.len() {
        h1[i] = s[2 * i].0;
        h2[i] = s[2 * i + 1].0;
    }
    (h1, h2)
}

fn calculate_Z(num: Vec<F3G>, den: Vec<F3G>) -> Vec<F3G> {
    let N = num.len();
    assert_eq!(N, den.len());
    let den_inv = F3G::batch_inverse(&den);
    let mut z = vec![F3G::ZERO; N];
    z[0] = F3G::ONE;
    for i in 1..N {
        z[i] = z[i - 1] * (num[i - 1] * den_inv[i - 1]);
    }

    let check_val = z[N - 1] * (num[N - 1] * den_inv[N - 1]);
    assert_eq!(check_val.eq(&F3G::one()), true);
    z
}

fn get_pol_ref<'a>(ctx: &'a mut StarkContext, starkinfo: &StarkInfo, id_pol: usize) -> Polynom<'a> {
    let p = &starkinfo.var_pol_map[id_pol];
    Polynom {
        buffer: ctx.get_mut(&p.section),
        deg: starkinfo.map_deg.get(&p.section),
        offset: p.section_pos,
        size: starkinfo.map_sectionsN.get(&p.section),
        dim: p.dim,
    }
}

pub fn get_pol(ctx: &mut StarkContext, starkinfo: &StarkInfo, id_pol: usize) -> Vec<F3G> {
    let p = get_pol_ref(ctx, starkinfo, id_pol);
    let mut res = vec![F3G::ZERO; p.deg];
    if p.dim == 1 {
        for i in 0..p.deg {
            res[i] = p.buffer[p.offset + i * p.size];
        }
    } else if p.dim == 3 {
        for i in 0..p.deg {
            res[i] = F3G::new(
                p.buffer[p.offset + i * p.size].to_be(),
                p.buffer[p.offset + i * p.size + 1].to_be(),
                p.buffer[p.offset + i * p.size + 2].to_be(),
            );
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
    res
}

pub fn extend_and_merkelize<M: MerkleTree>(
    ctx: &mut StarkContext,
    starkinfo: &StarkInfo,
    section_name: &'static str,
) -> Result<M> {
    let nBitsExt = ctx.nbits_ext;
    let nBits = ctx.nbits;
    let n_pols = starkinfo.map_sectionsN.get(section_name);
    let mut result = vec![F3G::ZERO; (1 << nBitsExt) * n_pols];
    let p = ctx.get_mut(section_name);
    interpolate(p, n_pols, nBits, &mut result, nBitsExt);
    let mut p_be = vec![FGL::ZERO; result.len()];
    p_be.par_iter_mut()
        .zip(result)
        .for_each(|(be_out, f3g_in)| {
            *be_out = f3g_in.to_be();
        });
    let mut tree = M::new();
    tree.merkelize(p_be, n_pols, 1 << nBitsExt)?;
    Ok(tree)
}

pub fn merkelize<M: MerkleTree>(
    ctx: &mut StarkContext,
    starkinfo: &StarkInfo,
    section_name: &'static str,
) -> Result<M> {
    let nBitsExt = ctx.nbits_ext;
    let n_pols = starkinfo.map_sectionsN.get(section_name);
    let p = ctx.get_mut(section_name);
    let mut p_be = vec![FGL::ZERO; p.len()];
    p_be.par_iter_mut().zip(p).for_each(|(be_out, f3g_in)| {
        *be_out = f3g_in.to_be();
    });
    let mut tree = M::new();
    tree.merkelize(p_be, n_pols, 1 << nBitsExt)?;
    Ok(tree)
}

pub fn calculate_exps(
    ctx: &mut StarkContext,
    starkinfo: &StarkInfo,
    seg: &Segment,
    dom: &str,
    step: &str,
    N: usize,
) {
    ctx.tmp = vec![F3G::ZERO; seg.tmp_used];
    let c_first = compile_code(ctx, starkinfo, &seg.first, dom, false);
    log::debug!(
        "calculate_exps compile_code {} ctx.first:\n{}",
        step,
        c_first
    );

    /*
    let mut N = if dom == "n" { ctx.N } else { ctx.Next };
    let _c_i = compile_code(ctx, starkinfo, &seg.i, dom, false);
    let _c_last = compile_code(ctx, starkinfo, &seg.last, dom, false);
    let next = if dom =="n" { 1 } else { 1<< (ctx.nBitsExt - ctx.nBits) };
    */
    // 0 ~ next: c_first
    // next ~ N-next: c_i
    // N-next ~ N: c_last
    for i in 0..N {
        c_first.eval(ctx, i);
        if (i % 10000) == 0 {
            log::info!("Calculating expression.. {}/{}", i, N);
        }
    }
}

pub fn calculate_exps_parallel(
    ctx: &mut StarkContext,
    starkinfo: &StarkInfo,
    seg: &Segment,
    _dom: &str,
    step: &str,
) {
    #[derive(Debug)]
    struct ExecItem {
        name: String,
        width: usize,
    }

    #[derive(Debug)]
    struct ExecInfo {
        input_sections: Vec<ExecItem>,
        output_sections: Vec<ExecItem>,
    }

    let mut exec_info = ExecInfo {
        input_sections: vec![],
        output_sections: vec![],
    };

    let dom = match step {
        "step2prev" => {
            exec_info.input_sections.push(ExecItem {
                name: "cm1_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "const_n".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "cm2_n".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "cm3_n".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "tmpexp_n".to_string(),
                width: 0,
            });
            "n"
        }
        "step3prev" => {
            exec_info.input_sections.push(ExecItem {
                name: "cm1_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm2_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm3_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "const_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "x_n".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "cm3_n".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "tmpexp_n".to_string(),
                width: 0,
            });
            "n"
        }
        "step3" => {
            exec_info.input_sections.push(ExecItem {
                name: "cm1_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm2_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm3_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "const_n".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "x_n".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "cm3_n".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "tmpexp_n".to_string(),
                width: 0,
            });
            "n"
        }
        "step4" => {
            exec_info.input_sections.push(ExecItem {
                name: "cm1_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm2_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm3_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "const_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "x_2ns".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "q_2ns".to_string(),
                width: 0,
            });
            "2ns"
        }
        "step5" => {
            exec_info.input_sections.push(ExecItem {
                name: "cm1_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm2_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm3_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "cm4_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "const_2ns".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "xDivXSubXi".to_string(),
                width: 0,
            });
            exec_info.input_sections.push(ExecItem {
                name: "xDivXSubWXi".to_string(),
                width: 0,
            });
            exec_info.output_sections.push(ExecItem {
                name: "f_2ns".to_string(),
                width: 0,
            });
            "2ns"
        }
        _ => panic!("Invalid step {}", step),
    };

    let set_width = |section: &mut ExecItem| {
        let name: &str = section.name.as_str();
        if name == "const_n" || name == "const_2ns" {
            section.width = starkinfo.n_constants;
        } else if starkinfo.map_sectionsN.get(name) != usize::MAX {
            section.width = starkinfo.map_sectionsN.get(name);
        } else if vec!["x_n", "x_2ns"].contains(&name) {
            section.width = 1;
        } else if vec!["xDivXSubXi", "xDivXSubWXi", "f_2ns"].contains(&name) {
            section.width = 3;
        } else if vec!["q_2ns"].contains(&name) {
            section.width = starkinfo.q_dim;
        } else {
            panic!("Invalid section name {}", name)
        }
    };

    for i in 0..exec_info.input_sections.len() {
        set_width(&mut exec_info.input_sections[i]);
    }
    for i in 0..exec_info.output_sections.len() {
        set_width(&mut exec_info.output_sections[i]);
    }

    let extend_bits = ctx.nbits_ext - ctx.nbits;
    let n = if dom == "n" { ctx.N } else { ctx.Next };
    let next = if dom == "n" { 1 } else { 1 << extend_bits };

    let mut n_per_thread = (n - 1) / get_max_workers() + 1;
    if n_per_thread > MAX_OPS_PER_THREAD {
        n_per_thread = MAX_OPS_PER_THREAD
    };
    if n_per_thread < MIN_OPS_PER_THREAD {
        n_per_thread = MIN_OPS_PER_THREAD
    };

    let mut ctx_chunks: Vec<StarkContext> = vec![];

    for i in (0..n).step_by(n_per_thread) {
        let cur_n = std::cmp::min(n_per_thread, n - i);
        let mut tmp_ctx = StarkContext::default();
        tmp_ctx.N = n;
        tmp_ctx.Next = next;
        tmp_ctx.nbits = ctx.nbits;
        tmp_ctx.nbits_ext = ctx.nbits_ext;
        tmp_ctx.evals = ctx.evals.clone();
        tmp_ctx.publics = ctx.publics.clone();
        tmp_ctx.challenge = ctx.challenge.clone();

        for si in &exec_info.input_sections {
            if si.name.as_str() == "xDivXSubXi" || si.name.as_str() == "xDivXSubWXi" {
                let tmp = tmp_ctx.get_mut_base(si.name.as_str());
                // for GL(p)
                *tmp = vec![FGL::ZERO; (cur_n + next) * si.width];
                let ori_sec = ctx.get_mut_base(si.name.as_str());
                for j in 0..(cur_n * si.width) {
                    tmp[j] = ori_sec[i * si.width + j]
                }
                // next
                for j in 0..(next * si.width) {
                    tmp[cur_n * si.width + j] = ori_sec[((i + cur_n) % n) * si.width + j]
                }
            } else {
                let tmp = tmp_ctx.get_mut(si.name.as_str());
                // for field extension GL(p^3)
                *tmp = vec![F3G::ZERO; (cur_n + next) * si.width];
                let ori_sec = ctx.get_mut(si.name.as_str());
                for j in 0..(cur_n * si.width) {
                    tmp[j] = ori_sec[i * si.width + j]
                }
                // next
                for j in 0..(next * si.width) {
                    tmp[cur_n * si.width + j] = ori_sec[((i + cur_n) % n) * si.width + j]
                }
            }
        }
        ctx_chunks.push(tmp_ctx);
    }

    ctx_chunks
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, tmp_ctx)| {
            let cur_n = std::cmp::min(n_per_thread, n - i * n_per_thread);
            log::info!("execute trace LDE {}/{}", i * n_per_thread, n);
            tmp_ctx.Zi = build_Zh_Inv(ctx.nbits, extend_bits, i * n_per_thread);
            for so in &exec_info.output_sections {
                let tmp = tmp_ctx.get_mut(so.name.as_str());
                if tmp.len() == 0 {
                    *tmp = vec![F3G::ZERO; so.width * (cur_n + next)];
                }
            }
            calculate_exps(tmp_ctx, starkinfo, seg, &dom, step, cur_n);
        });

    // write back the output
    for i in 0..ctx_chunks.len() {
        for so in &exec_info.output_sections {
            let tmp = ctx_chunks[i].get_mut(so.name.as_str());
            let out = ctx.get_mut(so.name.as_str());
            for k in 0..(tmp.len() - so.width * next) {
                out[i * n_per_thread * so.width + k] = tmp[k];
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::field_bn128::Fr;
    use crate::merklehash::MerkleTreeGL;
    use crate::merklehash_bn128::MerkleTreeBN128;
    use crate::polsarray::{PolKind, PolsArray};
    use crate::stark_gen::StarkProof;
    use crate::stark_setup::StarkSetup;
    use crate::stark_verify::stark_verify;
    use crate::traits::MTNodeType;
    use crate::transcript::TranscriptGL;
    use crate::transcript_bn128::TranscriptBN128;
    use crate::types::load_json;
    use crate::types::{StarkStruct, PIL};

    #[test]
    fn test_stark_gen() {
        env_logger::init();
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();

        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/fib.cm").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let mut setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let fr_root: Fr = setup.const_root.as_bn128();
        log::info!("setup {}", fr_root);
        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            &cm_pol,
            &const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
        )
        .unwrap();

        log::info!("verify the proof...");

        let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
            &starkproof,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &mut setup.program,
        )
        .unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_stark_permutation() {
        let mut pil = load_json::<PIL>("data/pe.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/pe.const").unwrap();

        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/pe.cm").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let mut setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            &cm_pol,
            &const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
        )
        .unwrap();

        log::info!("verify the proof...");

        let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
            &starkproof,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &mut setup.program,
        )
        .unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_stark_plookup_bn128() {
        let mut pil = load_json::<PIL>("data/plookup.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/plookup.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/plookup.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let mut setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            &cm_pol,
            &const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
        )
        .unwrap();
        log::info!("verify the proof...");
        let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
            &starkproof,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &mut setup.program,
        )
        .unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_stark_connection() {
        let mut pil = load_json::<PIL>("data/connection.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/connection.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/connection.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let mut setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            &cm_pol,
            &const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
        )
        .unwrap();
        log::info!("verify the proof...");
        let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
            &starkproof,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &mut setup.program,
        )
        .unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_stark_plookup_gl() {
        let mut pil = load_json::<PIL>("data/plookup.pil.json.gl").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/plookup.const.gl").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/plookup.cm.gl").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.gl").unwrap();
        let mut setup =
            StarkSetup::<MerkleTreeGL>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let starkproof = StarkProof::<MerkleTreeGL>::stark_gen::<TranscriptGL>(
            &cm_pol,
            &const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
        )
        .unwrap();
        log::info!("verify the proof...");
        let result = stark_verify::<MerkleTreeGL, TranscriptGL>(
            &starkproof,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &mut setup.program,
        )
        .unwrap();
        assert_eq!(result, true);
    }
}
