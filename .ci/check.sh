#!/usr/bin/env bash

ASM=`ls | grep *.asm`

function do_cargo_run()
{
	for item in $ASM; do
        cargo run $item
	done
}

OBJ=`ls | grep *.obj`

function do_cmp_obj()
{
	for item in $OBJ; do
        echo "do cmp obj: $item"
        cmp $item .ci/check-list/$item || exit 1
	done
}

set -x
do_cargo_run
do_cmp_obj