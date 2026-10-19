# Disponibilidade de broker, client e admin no painel

O A25 acrescenta três indicadores na seção **Visão geral** do XMQR Admin.
Confira o registro de aceite e a versão publicada: as instalações anteriores
não passam a mostrar esses indicadores apenas ao atualizar configurações.

| Indicador | Quando mostra ON-LINE | Limite da informação |
| --- | --- | --- |
| Broker | O monitor respondeu validamente e informou que o broker está pronto | Um monitor indisponível não comprova que o processo MQTT morreu |
| Client | A assinatura compartilhada do console recebeu confirmação SUBACK | Não enumera outros clients, processos externos ou ESP32 |
| Admin | O navegador recebeu uma resposta válida da API de status local | Carregar o HTML ou ter o programa instalado não confirma disponibilidade |

Antes da primeira leitura, o painel mostra **Aguardando**. Os estados usam
texto e cor. Se o navegador perder contato com o admin, o indicador do admin
fica OFF-LINE e os dois dependentes ficam sem confirmação. Isso evita mostrar
uma leitura antiga como se ainda fosse atual.

## Como utilizar

1. Inicie o broker com seu monitor habilitado e o admin conforme os guias de
   [console MQTT](mqtt-board.md) e [autenticação web](web-auth.md).
2. Abra `http://127.0.0.1:9080` no navegador do Windows e entre no painel.
3. Na Visão geral, confira os três indicadores e a explicação abaixo deles.
   O painel consulta os dados a cada ciclo de atualização existente, de três
   segundos, sem sobrepor as consultas. Também é possível atualizar a leitura
   pelo botão do painel.
4. Para o indicador Client ficar ON-LINE, conecte uma assinatura no Console
   MQTT e aguarde a confirmação do broker. O pedido de conexão sozinho ainda
   significa OFF-LINE, com a indicação de que a assinatura está conectando.
5. Desconectar a assinatura muda o Client para OFF-LINE. Um erro do processo
   também aparece no indicador; ele não encerra o serviço do broker.

Não houve mudança de portas: 9080 é o painel; 9090 é o monitor, incluindo
`/ready`; 8883 é o listener MQTT com TLS usado nos exemplos. O endereço de
saúde `http://127.0.0.1:9090/ready` continua sendo uma resposta HTTP curta,
sem interface gráfica.

## O significado das falhas

- **Broker não pronto:** o monitor respondeu, mas sua verificação de prontidão
  não passou. Consulte os logs do broker antes de tentar conectar dispositivos.
- **Monitor indisponível:** não houve leitura válida do monitor. Confira o
  endereço configurado, o serviço e seus logs. O MQTT pode continuar atendendo;
  o painel não tem confirmação suficiente para afirmar isso.
- **Client desconectado ou desabilitado:** não há assinatura ativa do console,
  ou os perfis ainda não foram configurados. Outros programas MQTT não são
  representados por esse indicador.
- **Admin inacessível, resposta inválida, timeout ou rede offline:** os estados
  anteriores são invalidados. Verifique se o admin continua em execução e se
  o navegador usa o endereço correto. A recuperação exige nova leitura válida.

Uma falha isolada ao carregar a lista de mensagens do console não substitui
um agregado de disponibilidade válido. Ao sair do painel ou trocar de conta,
os estados são limpos e uma nova sessão precisa obter seus próprios dados.

## Acesso de observadores

Usuários `viewer` podem consultar os indicadores e as métricas agregadas.
Não recebem o tópico, os payloads, as credenciais ou os caminhos dos perfis
do console. O indicador Client identifica apenas o estado da assinatura
compartilhada; sua leitura não renova o lease de 60 segundos dessa assinatura.
Para mantê-la em uso, o console autorizado deve continuar consultando sua
própria API. Uma tela de observação de métricas não substitui esse uso.

## Contrato de status

`GET /api/status` continua autenticado e mantém os campos anteriores:
`admin_version`, `reachable`, `ready`, `error` e `metrics`. Acrescenta
`systems.admin`, `systems.broker` e `systems.client`, cada um contendo
somente `online` booleano e `reason` textual estável.

| Componente | Valores de `reason` |
| --- | --- |
| Admin | `available` |
| Broker | `ready`, `not_ready`, `monitor_unavailable` |
| Client | `connected`, `connecting`, `disconnected`, `disabled`, `error` |

A resposta não representa um inventário de softwares instalados e não
certifica o caminho completo ESP32 → MQTT → API → banco → Google Sheets.
Essas etapas precisam de suas próprias verificações.

## Validação reproduzível

No Ubuntu/WSL, com um executável verificado:

```bash
python3 scripts/verify_system_status.py \
  --admin /caminho/absoluto/xmqr-admin \
  --report /var/tmp/xmqr-admin-a25-novo.json
```

O ensaio usa o admin real em portas temporárias, um monitor HTTP próprio e
um processo client determinístico. Verifica autenticação, acesso restrito do
viewer, monitor pronto/não pronto/indisponível, transição por SUBACK,
desconexão, falha e expiração do lease apesar de consultas ao agregado.
Leva mais de um minuto por incluir a espera real do lease de 60 segundos.
Não toca nos serviços, credenciais, perfis ou dados do operador. Um relatório
existente nunca é sobrescrito, inclusive quando seu resultado é FAIL.

O QA de navegador usa `browser_fixture_a25.py` com uma instância própria e
`browser_qa_a25.mjs` com o Chromium/Playwright já disponível no ambiente.
Ele separa as leituras reais da fixture das falhas injetadas no navegador:
JSON inválido, HTTP 500, atraso, indisponibilidade da rede e resposta anterior
chegando após um evento offline. Também verifica a recuperação, a invalidação
imediata com a API MQTT pendente, logout e layout mobile sem overflow.

Essas fixtures validam a composição de estados e a interface. O processo
determinístico não faz uma conexão MQTT real nem substitui o ensaio mTLS
broker/client do A23. Os resultados, hashes e screenshots devem constar no
registro A25, junto dos gates Rust e das regressões de autenticação A24.
