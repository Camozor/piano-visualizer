{
  description = "Piano visualizer";

  inputs.nixpkgs.url = "nixpkgs/nixos-26.05";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      defaultPackage.${system} = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustfmt
          rustc
          clippy
          pkg-config
          just
        ];
      };
    };
}
