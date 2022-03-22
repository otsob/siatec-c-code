#!/bin/sh

export POLY_TYPE=polyphonic
./run_on_jku_pdd.sh SIATEC
./run_on_jku_pdd.sh SIATEC-C --max-ioi=4.0

./run_on_jku_pdd.sh COSIATEC
./run_on_jku_pdd.sh SIATECCompress

./run_on_jku_pdd.sh COSIATEC-C --max-ioi=4.0
./run_on_jku_pdd.sh SIATEC-CCompress --max-ioi=4.0


export POLY_TYPE=monophonic
./run_on_jku_pdd.sh SIATEC
./run_on_jku_pdd.sh SIATEC-C --max-ioi=4.0

./run_on_jku_pdd.sh COSIATEC
./run_on_jku_pdd.sh SIATECCompress

./run_on_jku_pdd.sh COSIATEC-C --max-ioi=4.0
./run_on_jku_pdd.sh SIATEC-CCompress --max-ioi=4.0
