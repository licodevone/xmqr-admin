# A12 — alinhar core do admin independente ao broker 0.23.0

Entrada: admin independente 0.4.0 usa core Git pinado no broker 0.15.x (`3606ede...`), enquanto broker publicado 0.23.0 usa documento durável v10. O admin faz backup/verify/restore offline e valida bundle usando esse core. Este prompt precede qualquer mudança A12.

Objetivo: atualizar exclusivamente o pin imutável da dependência `mqtt-broker` para o commit exato da tag publicada `v0.23.0` (`b884a919f44a3a14c53b973b8a3f3e0aabc663e3`), mantendo o admin com versão própria 0.4.0 durante desenvolvimento. Ajustar integração apenas se APIs do core exigirem, sem alterar semântica de edição privada, canal local ou painel somente leitura. Verificar backup/verify/restore offline de estado v10 gerado por broker 0.23.0, inclusive negativo de backup inválido e sem abrir WAL de broker ativo.

Aceite: `cargo fmt/test/clippy/build --locked`, inventário de licenças, testes de distribuição, integração offline v10 com diretórios descartáveis, regressões de bundle/canal/painel e CI. Documentar escopo e limites. Checklist do admin somente após CI; tag/release própria apenas em marco de distribuição posterior. Sem tocar instalação WSL do operador, serviços, firewall ou segredos reais.

Revisão rev2 após primeiro ensaio: o backup durável do core não inclui `.mqtt-broker-profile`, deliberadamente externo ao WAL. O broker impede reabrir estado restaurado sem marcador em `open-lab` para não habilitar um listener inseguro por engano. O teste deve provar essa recusa e, apenas no diretório descartável, copiar explicitamente o marcador original `open-lab` antes de reabrir. O admin não criará nem copiará marcadores automaticamente; documentar o passo operacional e não declarar rollback automático do perfil.
