{
  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          nasm
        ];

        RUST_BACKTRACE=1;
      };
    };

  inputs.nixpkgs.url = "github:NixOs/nixpkgs/nixos-unstable";
}
