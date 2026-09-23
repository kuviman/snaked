{
  inputs = {
    geng.url = "github:geng-engine/cargo-geng";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    geng.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = inputs: (
    let
      system = "x86_64-linux";
      pkgs = import inputs.nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = inputs.geng.lib.mkShell {
        inherit system;
        target.linux.enable = true;
        target.web.enable = true;
        packages = with pkgs; [
          just
          butler
        ];
      };
    }
  );
}
