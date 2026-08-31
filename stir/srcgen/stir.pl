#!/usr/bin/env perl

use strict;
use warnings;
use FindBin qw($Bin);

require "$Bin/utils/printers.pl";

# Meta-constants: Rust types used in ISA definition
my $TY = "IRType";
my $VAL = "IRValue";
my $INSTR = "IRInstr";
my $BB = "IRBB";

# templates
my $binOp = {
  args => [ "ty:$TY", "dst:$VAL", "lhs:$VAL", "rhs:$VAL" ],
  uses => [ "lhs", "rhs" ],
  defs => [ "dst" ],
	fmt  => "{dst} = {name} {ty}, {lhs}, {rhs}",
};

my $resizeOp = {
  args => [ "to_ty:$TY", "dst:$VAL", "from_ty:$TY", "rs1:$VAL" ],
  uses => [ "rs1" ],
  defs => [ "dst" ],
  fmt => "{dst} = {name} {from_ty} {rs1} to {to_ty}",
};

# isa spec
my $isa = {
  isaName => "Stir",
  instrName => $INSTR,
  instructions => {
    comment => {
      args => [ "s:String" ],
      fmt => "; {s}"
    },

    add  => $binOp,
    sub  => $binOp,
    umul => $binOp,
    smul => $binOp,
    udiv => $binOp,
    sdiv => $binOp,

    sext  => $resizeOp,
    zext  => $resizeOp,
    trunc => $resizeOp,

    alloca => {
      args => [ "ty:$TY", "dst:$VAL" ],
      defs => [ "dst" ],
      fmt => "{dst} = alloca {ty}",
    },

    getaddr => {
      args => [ "dst:$VAL", "base:$VAL", "elem_ty:$TY", "idx:$VAL" ],
      uses => [ "base", "idx" ],
      defs => [ "dst" ],
      fmt => "{dst} = getaddr {base} offset by {elem_ty}, {idx}",
    },

    copy => {
      args => [ "ty:$TY", "dst:$VAL", "rs1:$VAL" ],
      uses => [ "rs1" ],
      defs => [ "dst" ],
      fmt  => "{dst} = copy {ty}, {rs1}",
    },

    load => {
      args => [ "ty:$TY", "ptr:$VAL", "dst:$VAL" ],
      uses => [ "ptr" ],
      defs => [ "dst" ],
      fmt  => "{dst} = load {ty} from {ptr}",
    },

    store => {
      args => [ "ty:$TY", "ptr:$VAL", "rs1:$VAL" ],
      uses => [ "ptr", "rs1" ],
      fmt  => "store {ty} {rs1} into {ptr}",
    },

    icmp => {
      args => [ "cmp:CmpOp", "ty:$TY", "dst:$VAL", "lhs:$VAL", "rhs:$VAL" ],
      uses => [ "lhs", "rhs" ],
      defs => [ "dst" ],
      fmt  => "{dst} = {name} {cmp} {ty}, {lhs}, {rhs}",
    },

    br => {
      args => [ "val:$VAL", "truebb:$BB", "falsebb:$BB" ],
      uses => [ "val" ],
      term => 1,
    },

    jmp => {
      args => [ "to:$BB" ],
      term => 1,
    },

    ret => { 
      args => [ "ty:$TY", "val:$VAL" ],
      uses => [ "val" ],
      term => 1,
    },

    retv => {
      term => 1,
    },
  },
  value => $VAL,

  extraCode => [
    "use crate::target::stir::isa::*;",
    "use crate::target::stir::builder::*;",
  ],

  specFile => __FILE__,
};

printAll($isa);
