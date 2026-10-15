# Histórico

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
