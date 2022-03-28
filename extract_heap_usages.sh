#!/bin/sh

for file in ./output/heaptrack_output/**/*.gz; 
do
   echo "Heap usage with:" $file
   heaptrack --analyze $file | grep "peak heap memory consumption"
   echo "----------------"
done
