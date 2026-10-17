# A19 rev2 — compatibilidade do admin independente com estado v15

Entrada: P164 adicionará intervalo de Message Expiry ao Will pendente e migrará o documento durável de v14 para v15. O admin 0.9.0 fixa broker v14. Este prompt precede a atualização do pin.

Objetivo: fixar o commit exato do broker P164, regenerar lockfile e verificar que o admin independente faz backup, verifica e restaura em destino novo o documento v15 sem perder Wills pendentes ou metadados. Preservar os testes para formatos anteriores e a separação entre estado de broker/admin. O admin 0.9.0 publicado consegue transportar backup bruto de v15 sem interpretar o documento; isso não homologa o par antigo. O broker 0.28.0 rejeita a leitura de v15 após a primeira gravação, de modo que rollback executável requer backup privado v14 anterior à migração.

Aceite: suíte Rust e integração real do admin com broker v15, caso positivo com Will pendente de Message Expiry, prova de backup bruto no admin 0.9.0 e recusa do broker 0.28.0 ao abrir v15; fmt/Clippy/release/licenças/distribuição/CI exata. Só então considerar nova tag/release conjunta no marco apropriado.
