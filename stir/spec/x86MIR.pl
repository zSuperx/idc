#!/usr/bin/env perl

use strict;
use warnings;
use FindBin qw($Bin);

require "$Bin/utils/printers.pl";

# Meta-constants: Rust types used in ISA definition
my $VAL = "x86Val";
my $INSTR = "x86Instr";
my $BB = "BBID";

# templates
my $accumOp = {
  args => [ "rs1:$VAL", "rs2:$VAL" ],
  uses => [ "rs1", "rs2" ],
  defs => [ "rs1" ],
	fmt  => "{name} {rs1}, {rs2}",
  term => 0,
};

my $move = {
  args => [ "dst:$VAL", "rs1:$VAL" ],
  uses => [ "rs1" ],
  defs => [ "dst" ],
  fmt => "{name} {dst}, {rs1}",
};

# isa spec
my $isa = {
  isaName => "x86",
  instrName => $INSTR,
  value => $VAL,
  specFile => __FILE__,
  extraCode => [
    "use super::*;"
  ],

  instructions => {
    add  => $accumOp,
    sub  => $accumOp,
    imul => $accumOp,
    idiv => $accumOp,

    jcc => {
      args => [ "to$BB", "flag:RFLAG" ],
      uses => [ "flag" ],
      term => 1,
      fmt  => "JCC {to}, {flag}",
    },

    jmp => { term => 1 },
  },
};

printAll($isa);
