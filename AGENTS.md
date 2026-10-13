# XMQR Admin — instruções

Projeto independente, não workspace: binário xmqr-admin, versão própria.
Antes de editar, preparar/presentar prompt em prompts/admin, registrar estado,
escopo/arquivos, compatibilidade, negativos e aceite. Registro em prompts/registros.
Código próprio MIT; preserve notices e licenças de dependências. CoreGit pinado
por commit, sem pathdependency nem alteração automática do broker/client.

Não remover mqtt-admin legado. PainelA02 é leitura de agregados via monitor
loopback, não controle de sessões/ACL. Não abrir WAL enquanto broker ativo nem
inferir segurança/conformidade/produção por testes unitários. CLI CRUD é legado
lab/Unix, backup offline; preservar locks/symlink/0600/rename/fsync e formatos.

Não publicar token, senhas, chaves, certificados ou estado real. Tokenweb não é
senhaMQTT, permanece em memória do browser. Host/Origin/Bearer/no-store/CSP,
loopback, bodylimits/deadlines e concorrência devem ter negativos reais.
Sem mudanças no host/serviços/firewall/firmware. Testes somente recursos próprios.

Gates: cargo fmt --all -- --check; cargo test --locked;
cargo clippy --locked --all-targets -- -D warnings; cargo build --locked --release;
python3 scripts/license_inventory.py --check; integração/browser proporcionais.
Linux/WSL é alvo de estado/token; Windows nativo ainda não homologado.

Pedido vigente autoriza commit/push, tags de código aceito e release/.deb/Docker
após gates. Sem force/reset/overwrite/latest. Tags imutáveis; docs sozinhas não
mudam versão. Milestones posteriores usam novo prompt; não executar backlog inteiro.
