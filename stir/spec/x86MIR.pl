#!/usr/bin/env perl

use strict;
use warnings;
use FindBin qw($Bin);

require "$Bin/utils/printers.pl";

# Meta-constants: Rust types used in ISA definition
my $VAL = "x86MIRValue";
my $TY = "IRType";
my $INSTR = "x86MIRInstr";
my $BB = "BBID";

# templates
my $accumOp = {
  args => [ "rs1:$VAL", "rs2:$VAL" ],
  uses => [ "rs1", "rs2" ],
  defs => [ "rs1" ],
	fmt  => "{name}\t {rs1}, {rs2}",
  term => 0,
};

my $move = {
  args => [ "dst:$VAL", "rs1:$VAL" ],
  uses => [ "rs1" ],
  defs => [ "dst" ],
  fmt => "{name}\t {dst}, {rs1}",
};

# isa spec
my $isa = {
  isaName => "x86MIR",
  instrName => $INSTR,
  value => $VAL,
  specFile => __FILE__,
  extraCode => [
    "use super::*;"
  ],

  instructions => {
    comment => {
      args => [ "s:String" ],
      fmt => "; {s}",
    },

    add  => $accumOp,
    sub  => $accumOp,
    imul => $accumOp,
    idiv => $accumOp,

    zext => {
      args => [ "dst:$VAL", "src:$VAL" ],
      uses => [ "src" ],
      defs => [ "dst" ],
      fmt  => "movzx\t {dst}, {src}",
    },

    sext => {
      args => [ "dst:$VAL", "src:$VAL" ],
      uses => [ "src" ],
      defs => [ "dst" ],
      fmt  => "movsx\t {dst}, {src}",
    },

    copy => {
      args => [ "dst:$VAL", "src:$VAL" ],
      uses => [ "src" ],
      defs => [ "dst" ],
      fmt  => "COPY\t {dst}, {src}",
    },

    frameslot => {
      args => [ "dst:$VAL" ],
      defs => [ "dst" ],
      fmt  =>  "FSLOT\t {dst}",
    },

    load => {
      args => [ "dst:$VAL", "src:$VAL" ],
      uses => [ "src" ],
      defs => [ "dst" ],
      fmt  => "LOAD\t {dst}, {src}",
    },

    store => {
      args => [ "dst:$VAL", "src:$VAL" ],
      uses => [ "src" ],
      defs => [ "dst" ],
      fmt  => "STORE\t {dst}, {src}",
    },

    jcc => {
      args => [ "to:$BB", "flag:$VAL" ],
      uses => [ "flag" ],
      term => 1,
      fmt  => "JCC\t {to}, {flag}",
    },

    jmp => { 
      args => [ "to:$BB" ],
      term => 1,
    },

    ret => {
      term => 1,
      fmt => "ret\t",
    },
  },
};

printAll($isa);
