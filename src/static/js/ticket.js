document.addEventListener("DOMContentLoaded", () => {
  const openTicketForm = document.getElementById("open_ticket_form");
  const logoutBtn = document.getElementById("logoutBtn");

  // Função assíncrona para carregar os tickets do utilizador logado.
  async function carregarMeusTickets() {
    const token = localStorage.getItem("token");
    const tbody = document.querySelector("table tbody");

    if (!tbody) {
      // Este erro é importante para o desenvolvimento, caso o seletor da tabela mude.
      console.error("Elemento tbody da tabela não encontrado no HTML!");
      return;
    }
    // Define uma mensagem inicial de carregamento na tabela.
    tbody.innerHTML =
      '<tr><td colspan="7" style="text-align:center;">Carregando seus chamados...</td></tr>';

    if (!token) {
      // Se não houver token, exibe mensagem e não prossegue.
      // O script no <head> do HTML já deve ter redirecionado para o login.
      tbody.innerHTML =
        '<tr><td colspan="7" style="text-align:center;">Sessão não encontrada. Por favor, <a href="/login">faça login</a>.</td></tr>';
      return;
    }

    try {
      // Realiza a chamada API para buscar os tickets do utilizador.
      const response = await fetch("/new_ticket/api/my-open-tickets", {
        method: "GET",
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
      });

      if (!response.ok) {
        // Trata respostas não bem-sucedidas da API.
        if (response.status === 401) {
          alert(
            "Sessão inválida ou expirada. Por favor, faça login novamente."
          );
          localStorage.removeItem("token");
          window.location.href = "/login";
          return;
        }
        const errorData = await response.json().catch(() => ({})); // Tenta obter detalhes do erro.
        throw new Error(
          errorData.erro ||
            `Erro ao carregar seus chamados (Status: ${response.status})`
        );
      }

      const tickets = await response.json();
      tbody.innerHTML = ""; // Limpa a tabela antes de adicionar os novos dados.

      if (tickets.length === 0) {
        tbody.innerHTML =
          '<tr><td colspan="7" style="text-align:center;">Você não possui chamados abertos.</td></tr>';
      } else {
        // Itera sobre os tickets recebidos e os adiciona à tabela.
        tickets.forEach((ticket) => {
          const tr = document.createElement("tr");
          tr.innerHTML = `
            <td>${ticket.ticket_id || "N/A"}</td>
            <td>${ticket.ticket_title || ""}</td>
            <td>${ticket.ticket_status || ""}</td>
            <td>${ticket.ticket_priority || ""}</td>
            <td>${ticket.ticket_client_name || "N/A"}</td>
            <td>${ticket.ticket_category || ""}</td>
            <td>
              <button class="btn btn-sm btn-outline-warning fw-bold close-btn" data-id="${
                ticket.ticket_id
              }">
                Fechar
              </button>
            </td>
          `;
          tbody.appendChild(tr);
          // Adiciona o listener de evento para o botão "Fechar" de cada ticket.
          const closeButton = tr.querySelector(".close-btn");
          if (closeButton) {
            closeButton.addEventListener("click", closeTicketHandler);
          }
        });
      }
    } catch (error) {
      // Exibe uma mensagem de erro na tabela se a busca de tickets falhar.
      // Manter um console.error aqui pode ser útil para depurar erros de rede/API.
      console.error("Erro ao buscar ou renderizar tickets:", error);
      tbody.innerHTML = `<tr><td colspan="7" style="text-align:center;">Erro ao carregar seus chamados: ${error.message}</td></tr>`;
    }
  }

  // Adiciona listener para o formulário de abertura de ticket, se existir.
  if (openTicketForm) {
    openTicketForm.addEventListener("submit", async function (e) {
      e.preventDefault(); // Previne a submissão padrão do formulário.

      const titulo = document.getElementById("titulo").value;
      const descricao = document.getElementById("descricao").value;
      const categoria = document.querySelector(
        'input[name="ticket_category"]:checked'
      )?.value;

      if (!categoria) {
        alert("Por favor, selecione uma categoria.");
        return;
      }

      const token = localStorage.getItem("token");
      if (!token) {
        alert("Sessão expirada. Por favor, faça login novamente.");
        window.location.href = "/login";
        return;
      }

      const ticketParaEnviar = {
        ticket_title: titulo,
        ticket_description: descricao,
        ticket_category: categoria,
        ticket_client_id: 0, // O backend definirá o ID do cliente com base no JWT.
      };

      try {
        // Envia os dados do novo ticket para o backend.
        const response = await fetch("/new_ticket", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify(ticketParaEnviar),
        });

        if (!response.ok) {
          if (response.status === 401) {
            alert("Sessão inválida. Por favor, faça login novamente.");
            localStorage.removeItem("token");
            window.location.href = "/login";
            return;
          }
          const errorData = await response.json().catch(() => ({}));
          throw new Error(
            errorData.erro ||
              `Erro na criação do chamado (Status: ${response.status})`
          );
        }

        alert("Chamado criado com sucesso!");
        this.reset(); // Limpa os campos do formulário.
        carregarMeusTickets(); // Recarrega a lista de tickets para incluir o novo.
      } catch (error) {
        alert(error.message);
      }
    });
  }

  // Função para lidar com o fechamento de um ticket.
  async function closeTicketHandler(e) {
    // Garante que o clique foi no botão de fechar.
    if (!e.target.classList.contains("close-btn")) return;

    const button = e.target;
    const ticketId = button.dataset.id;

    const token = localStorage.getItem("token");
    if (!token) {
      alert("Sessão expirada. Por favor, faça login novamente.");
      window.location.href = "/login";
      return;
    }

    try {
      // Envia a requisição para fechar o ticket.
      const response = await fetch(`/tickets/${ticketId}/close`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        if (response.status === 401) {
          alert("Sessão inválida. Por favor, faça login novamente.");
          localStorage.removeItem("token");
          window.location.href = "/login";
          return;
        }
        throw new Error("Erro ao fechar chamado");
      }

      alert("Chamado fechado com sucesso!");
      carregarMeusTickets(); // Recarrega a lista de tickets para refletir a mudança.
    } catch (error) {
      alert(error.message);
    }
  }

  // Adiciona listener para o botão de logout, se existir.
  if (logoutBtn) {
    logoutBtn.addEventListener("click", () => {
      localStorage.removeItem("token");
      window.location.href = "/login";
    });
  }

  // Carrega os tickets do utilizador se um token estiver presente.
  if (localStorage.getItem("token")) {
    carregarMeusTickets();
  }
});
