# A05 — validação offline do bundle seguro

- Prompt: [A05 rev1–4](../admin/A05-validar-bundle-seguro.md).
- Entrada: `1c428bb`, admin 0.1.0; alterações parciais preservadas e concluídas.
- Plataforma: Ubuntu 26.04/WSL; core Git pinado em `3606ede` sem repin.
- Entrega: `xmqr-admin bundle validate --file ARQUIVO` chama DynamicPolicy
  do core pinado em modo seguro, sem iniciar broker, abrir WAL ou alterar arquivo.
  Saída de sucesso fixa e erros sanitizados.
- Gates finais: fmt, `cargo test --locked` (23 testes), Clippy `-D warnings`,
  build release e inventário de licenças PASS. Fixtures cobrem hash Argon2,
  fingerprint, referência inválida, modo público e symlink; conteúdo idêntico
  após validação e sem secret na saída.
- Revisão de segurança: política de arquivo privado/ownership/symlink/quotas
  reutilizada do validador do core; sem log do conteúdo nem caminho fornecido.
  Cobertura limitada a Linux/WSL e ao comando offline.
- Compatibilidade: novo comando opt-in; painel e CRUD legado inalterados. U02
  parcial; editor e aplicação do bundle continuam pendentes.
- Publicação: candidato à tag de código `v0.2.0` após revisão do diff.
