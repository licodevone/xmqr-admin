# A19 rev1 — compatibilidade do admin independente com estado v15

Entrada: P164 adicionará intervalo de Message Expiry ao Will pendente e migrará o documento durável de v14 para v15. O admin 0.9.0 fixa broker v14. Este prompt precede a atualização do pin.

Objetivo: fixar o commit exato do broker P164, regenerar lockfile e verificar que o admin independente lê, diagnostica, faz backup, verifica e restaura em destino novo o documento v15 sem perder Wills pendentes ou metadados. Preservar os testes para formatos anteriores e a separação entre estado de broker/admin. Documentar que admin antigo 0.9.0 não abre v15 após a primeira gravação e que rollback requer backup privado v14 anterior à migração.

Aceite: suíte Rust e integração real do admin com broker v15, caso positivo com Will pendente de Message Expiry, negativos de versão nova no admin antigo, fmt/Clippy/release/licenças/distribuição/CI exata. Só então considerar nova tag/release conjunta no marco apropriado.
