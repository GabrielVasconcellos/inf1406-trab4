# INF1406 - Trabalho 4 para a disciplina de Programação Distribuída Concorrente

Objetivo do trabalho: Implementação de Servidores Replicados usando Filas MQTT.

- Linguagem escolhida: Rust
- Crate: Mosquitto-Client (É necessário instalar a biblioteca libmosquitto-dev no Ubuntu)
- Broker: Mosquitto (Versão 2.0.10, encontramos alguns problemas com versões superiores a 2.0.11)
- Crate para Json: serde

Etapa inicial: Testar a linguagem e biblioteca escolhidas através de uma aplicação exemplo. ✓

Aspectos do sistema: 

Estrutura:
- São criados n servidores e um processo monitor.
- Cada servidor é um processo inscrito no tópico inf1406-reqs. 
- O monitor é um processo inscrito no tópico inf1406-mon.
- Cada um deles possui o seu próprio HashMap e um vetor log de tamanho n, que é atualizado em um tempo T.
- O tempo T é 10 vezes maior que o tempo usado para detecção de falha usado pelo monitor.
- Cada índice do vetor log possui todas as requisições feitas para o servidor de tal índice no heartbeat atual.

Funcionamento do servidor ao receber requests:
- É criada uma thread para lidar com cada request (insert ou query).
- Cada inserção é executada por todos os servidores.
- No caso de uma consulta, calcula-se a hash para descobrir quem é o servidor responsável por atendê-la.
- A hash é calculada de acordo com a fórmula: (Σ(i=1,#chave) i-ésimo byte da chave) mod n
- Existe uma thread específica para acessar o HashMap.
- Usando os canais do Rust, é feita a comunicação entre a thread que trata as requisições e a thread que lida com o HashMap.

Funcionamento do servidor para o tratamento de falhas:
- Envia-se um heartbeat para o monitor periodicamente, na thread principal dos servidores.
- No ínicio do heartbeat o servidor deve verificar se ele se tornou o substituto de outro servidor.
- A fórmula para definir o servidor substituto é: (idserv + 1) mod n, sendo idserv o id do server que falhou.
- Caso ele tenha se tornado um substituto, ele acessa o seu log para executar todas as queries destinadas ao servidor que falhou.
- Após terminar de executar todas as queries, o servidor está pronto para atender suas requests.

Funcionamento do monitor:
- Ao concluir que um processo falhou, o monitor publica uma mensagem no tópico inf1406-reqs.
- Após sinalizar que o processo falhou, o monitor spawna um novo processo passando os parâmetros corretos para inicializá-lo como um processo novo.

Explicar solução encontrada para a sincronização entre um servidor novo e substituto: Enquanto o servidor novo não recebe o HashMap atualizado do servidor substituto, ele não pode atender requests. Assim que o servidor substituto completar o envio, o servidor novo deverá publicar no tópico inf1406-reqs uma mensagem para informar que está pronto para receber suas requests. Ao receber esta mensagem, o servidor substituto para de responder as requests do servidor novo.

Dúvidas:

- O que acontece quando dois servidores falham?
- Seria possível dividir a tarefa de atualizar um servidor novo por todos os servidores?
- É o monitor que deve criar servidores novos após detectar falhas?
