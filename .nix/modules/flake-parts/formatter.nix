{inputs, ...}: {
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    treefmt.config = {
      projectRootFile = "flake.nix";
      programs.alejandra.enable = true;
      programs.taplo.enable = true;
      programs.rustfmt.enable = true;
    };
  };
}
