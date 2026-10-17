# Histórico

## Em desenvolvimento após 0.9.0

- A19: core do broker P164 fixado por commit imutável, com documento v15 e Will pendente que conserva Message Expiry Interval em backup/verify/restore. Admin 0.9.0 também transporta backup bruto v15, mas broker 0.28.0 não o abre; rollback executável exige backup privado v14 anterior.

## 0.9.0 — core broker 0.27.0 e estado v14 (experimental)

- A18: core Git imutável `53d86a7e296c158d9247bc5011dec1da49c39b29` do broker 0.27.0. Backup/verify/restore offline preserva origem de cópia QoS1 compartilhada redistribuída, Subscription Identifier e fila do membro sobrevivente. Perfil de listener permanece externo e explícito; painel continua somente leitura.
- Broker 0.26.0/admin 0.8.0 não reabrem v14. Rollback exige backup privado v13 anterior à primeira gravação no broker 0.27.0.

## 0.8.0 — core broker 0.26.0 e estado v13 (experimental)

- A16: core Git imutável `0254a556` da tag broker 0.26.0; backup/verify/restore offline do estado v13 preserva associação compartilhada durável, identificador de assinatura e entrega QoS2 escolhida. Perfil `.mqtt-broker-profile` externo e explícito; painel somente leitura. Broker 0.25.0/admin 0.7.0 não reabrem v13; rollback exige backup privado v12 anterior à migração.

## 0.7.0 — core broker 0.25.0 e estado v12 (experimental)

- A15: core Git imutável `6462b838` do broker publicado 0.25.0; backup/verify/restore offline de estado v12 preserva Subscription Identifier por assinatura. Perfil `.mqtt-broker-profile` externo e explícito; painel somente leitura. Broker 0.24.0/admin 0.6.0 não reabrem estado v12, que exige backup privado v11 anterior para rollback.

## 0.6.0 — core broker 0.24.0 e estado v11 (experimental)

- A14: core Git imutável `8dde5fda` do broker publicado 0.24.0; backup/verify/restore offline de estado v11 com No Local e Retain As Published persistidos. O perfil `.mqtt-broker-profile` permanece externo ao backup e deve ser selecionado explicitamente. Painel continua somente leitura.
- Pacote Ubuntu 26.04 amd64 e imagem numerada publicados após gates de distribuição A14. Broker 0.23.0 e admin 0.5.0 não reabrem estado v11; rollback de estado exige backup privado v10 anterior à migração.

## 0.5.0 — core broker 0.23.0 (experimental)

- A12: core Git do broker atualizado de `3606ede` (0.11.5) para a fonte exata publicada 0.23.0 (`b884a919`), com estado durável v10 e dependências/licenças revisadas. CLI, canal e painel preservados. Backup/restore continua offline; o perfil de listener deve ser escolhido explicitamente ao reabrir estado restaurado.
- A13: binário, pacote Ubuntu26 amd64 e imagem numerada passam a distribuir esse core após gates e publicação do marco. Painel somente leitura; sem promessa de migração de perfil de listener.

## 0.3.0 — editor offline do bundle seguro

- A06: papéis, grupos, ACLs diretas, vínculos e habilitação/revogação de
  identidades no bundle TOML v1. Cada candidato usa lock privado, validação
  pelo core, arquivo temporário 0600, sync e rename atômico.
- A edição não recarrega o broker nem abre WAL; o canal online A09 permanece
  pendente. Painel local e CLI legado preservados.

## 0.2.0 — validação offline do bundle seguro

- A05: `xmqr-admin bundle validate --file ARQUIVO` usa o validador do core
  pinado para conferir formato, referências, fingerprint e política de arquivo
  privado sem modificar o bundle ou o estado. Erros são sanitizados.
- É a primeira parcela de U02; edição de grupos, papéis e ACL ainda não existe.

## 0.1.0 — primeiro admin independente

- CLI de usuários/hash/fingerprint e backup/verify/restore preservada, com
  origem rastreável e biblioteca core pinada por commit.
- Painel local autenticado, agregado e somente leitura; monitor opcional real,
  disponibilidade/erro separados, sem dados fictícios ou payloads.
- Token privado, Host/Origin/Bearer, CSP/no-store, limites/deadlines e shutdown.
- Admissão TCP32 antes dos headers, vida absoluta10s e requisições16; token
  lido sem seguir symlink, metadados do próprio descritor e pai privado.
- Sem substituição do mqtt-admin legado, mudanças no broker/client ou gestão
  ao vivo. ExperimentalLinux/WSL; sem suporte remoto ou produção declarado.
