document.addEventListener("DOMContentLoaded", () => {
  const loginForm = document.getElementById("login"); // Obtém o formulário de login pelo ID.

  if (loginForm) {
    // Adiciona um listener para o evento de submissão do formulário.
    loginForm.addEventListener("submit", async (e) => {
      e.preventDefault(); // Previne o comportamento padrão de submissão do formulário.

      // Obtém os valores dos campos de email e senha.
      const email = document.getElementById("email").value;
      const password = document.getElementById("password").value;
      const loginObj = { email, password };

      // Envia os dados de login para o servidor.
      await sendLoginData(loginObj);
    });
  } else {
    // Este erro é importante manter, caso o ID do formulário no HTML esteja incorreto.
    console.error(
      "ERRO: Formulário com id='login' não foi encontrado no HTML."
    );
  }
});

// Função assíncrona para enviar os dados de login ao backend.
async function sendLoginData(login_obj) {
  const loginPath = window.location.origin + "/login"; // Constrói o URL do endpoint de login.

  try {
    // Realiza a requisição POST para o endpoint de login.
    const response = await fetch(loginPath, {
      method: "POST",
      headers: {
        "Content-Type": "application/json", // Informa ao servidor que o corpo é JSON.
      },
      body: JSON.stringify(login_obj), // Converte o objeto de login para uma string JSON.
    });

    const data = await response.json(); // Tenta converter a resposta do servidor para JSON.

    // Verifica se a requisição foi bem-sucedida (status 2xx) e se um token foi retornado.
    if (response.ok && data.token) {
      localStorage.setItem("token", data.token); // Armazena o token no localStorage.

      // Redireciona o utilizador com base no seu papel (role).
      if (data.role === "Administrador") {
        window.location.href = "/admin/dashboard"; // Redireciona administradores.
      } else if (data.role === "Cliente") {
        window.location.href = "/new_ticket"; // Redireciona clientes.
      } else {
        // Fallback se o papel não for reconhecido.
        alert(
          "Login bem-sucedido, mas não foi possível determinar a sua página inicial."
        );
        window.location.href = "/";
      }
    } else {
      // Se a resposta não for 'ok' ou não houver token, exibe uma mensagem de erro.
      const errorMessage =
        data.error || "Login falhou! Verifique as suas credenciais.";
      alert(errorMessage);
    }
  } catch (error) {
    // Captura erros na requisição fetch ou na conversão para JSON.
    // console.error original mantido para depuração de erros de rede/parse, mas pode ser removido se desejado.
    console.error("Erro na função sendLoginData:", error);
    alert("Erro ao conectar com o servidor durante o login!");
  }
}
