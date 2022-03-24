#!/bin/sh

# Usage
# - Set the path variables to correct paths
# - Run the script with: ./run_on_jku_pdd.sh <ALGORITHM> <Additional ARGs to posemir_cli>


##################################################################################
# Set these paths:

# Set this to polyphonic or monophonic to choose whether to use monophonic or polyphonic JKU-PDD
# POLY_TYPE=polyphonic

# Set this to absolute path to JKU-PDD groundTruth directory
JKU_PDD_PATH=./jku-pdd/groundTruth

# Set this to the path for the posemir_cli executable 
CLI_PATH=./posemir/target/release/posemir_cli

# Set this to the path where you want the output to be saved
OUTPUT_PATH=./output


##################################################################################
# Below this you should not have to modify anything.
ALGORITHM=$1

# Additional arguments for posemir_cli
ARGS=${@:2}

OUTPUT_DIR=${OUTPUT_PATH}/${ALGORITHM}_${POLY_TYPE}
rm -r $OUTPUT_DIR
mkdir -p $OUTPUT_DIR

# Bach
INPUT_PATH=${JKU_PDD_PATH}/bachBWV889Fg/${POLY_TYPE}/csv/wtc2f20.csv
PIECE=bachBWV889Fg_${POLY_TYPE}
${CLI_PATH} --algo ${ALGORITHM} -p ${PIECE} -i ${INPUT_PATH} -o ${OUTPUT_DIR} $ARGS
cp ${INPUT_PATH} ${OUTPUT_DIR}/${PIECE}.csv

# Beethoven
PIECE=beethovenOp2No1Mvt3_${POLY_TYPE}
INPUT_PATH=${JKU_PDD_PATH}/beethovenOp2No1Mvt3/${POLY_TYPE}/csv/sonata01-3.csv
${CLI_PATH} --algo ${ALGORITHM} -p ${PIECE} -i ${INPUT_PATH} -o ${OUTPUT_DIR} $ARGS
cp ${INPUT_PATH} ${OUTPUT_DIR}/${PIECE}.csv

# Chopin
PIECE=chopinOp24No4_${POLY_TYPE}
INPUT_PATH=${JKU_PDD_PATH}/chopinOp24No4/${POLY_TYPE}/csv/mazurka24-4.csv
${CLI_PATH} --algo ${ALGORITHM} -p ${PIECE} -i ${INPUT_PATH} -o ${OUTPUT_DIR} $ARGS
cp ${INPUT_PATH} ${OUTPUT_DIR}/${PIECE}.csv

# Gibbons
PIECE=gibbonsSilverSwan1612_${POLY_TYPE}
INPUT_PATH=${JKU_PDD_PATH}/gibbonsSilverSwan1612/${POLY_TYPE}/csv/silverswan.csv
${CLI_PATH} --algo ${ALGORITHM} -p ${PIECE} -i ${INPUT_PATH} -o ${OUTPUT_DIR} $ARGS
cp ${INPUT_PATH} ${OUTPUT_DIR}/${PIECE}.csv

# Mozart
PIECE=mozartK282Mvt2_${POLY_TYPE}
INPUT_PATH=${JKU_PDD_PATH}/mozartK282Mvt2/${POLY_TYPE}/csv/sonata04-2.csv
${CLI_PATH} --algo ${ALGORITHM} -p ${PIECE} -i ${INPUT_PATH} -o ${OUTPUT_DIR} $ARGS
cp ${INPUT_PATH} ${OUTPUT_DIR}/${PIECE}.csv
