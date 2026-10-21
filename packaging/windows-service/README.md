# Build experimental do serviço

Manifest independente, sem workspace, compartilhando `../../src/main.rs`.
Dependências normais, versão e core Git pinado devem acompanhar o Cargo.toml
da raiz; somente este manifest acrescenta windows-service e habilita
windows-service-runtime.

Obter dependências, gerar/revisar Cargo.lock nesta pasta e executar os gates
fmt/test/clippy/build --locked neste manifest antes de distribuir. O binário
válido responde a `--windows-service-capabilities` com JSON schema=1,
component=admin e service_entrypoint=true. O binário da raiz não é serviço.

Estado atual e limitações: [guia](../../docs/windows-service.md).
Nenhum Cargo.lock experimental foi fabricado durante indisponibilidade da rede.
