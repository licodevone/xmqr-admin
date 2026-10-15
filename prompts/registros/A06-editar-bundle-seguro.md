# A06 — editor offline do bundle seguro

- Prompt: [A06 rev1–2](../admin/A06-editar-bundle-seguro.md).
- Entrada: admin 0.2.0, core Git fixado em `3606ede`, broker 0.14.0.
- Entrega: CLI de papéis/grupos com ACLs, vínculos e ACLs diretas de usuário,
  habilitação/revogação, listagem sem hashes. Cada candidato é validado pelo
  `DynamicPolicy` do core, gravado em arquivo temporário privado, sincronizado
  e renomeado sob lock. Concorrência recusa segundo editor, sem espera infinita.
- Negativos: concessão inválida, referência ainda usada, arquivo público,
  symlink e lock ocupado preservam o bundle. Alterações são **offline**;
  nenhum reload, canal A09 ou WAL é aberto por este CLI.
- Ambiente: Linux/WSL; bundle existente em diretório que não permita escrita
  por grupo/outros. Windows nativo permanece sem aceite.
- Gates finais no Ubuntu 26.04/WSL: `cargo fmt --all -- --check`, 21 testes
  unitários + 6 de integração, Clippy `-D warnings`, build release e
  `license_inventory.py --check` PASS. Os testes usam bundles temporários
  públicos de fixture; nenhum serviço/estado do operador foi modificado.
