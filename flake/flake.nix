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
          mold
        ];
      };
    };

  inputs.nixpkgs.url = "github:NixOs/nixpkgs/nixos-unstable";
}
