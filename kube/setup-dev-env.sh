#!/bin/bash

# Interrompt le script en cas d'erreur
set -e

echo "--- 🚀 Début de la configuration de l'environnement de dev ---"

# 1. Mise à jour du système
echo "--- 📦 Mise à jour des paquets ---"
sudo apt-get update && sudo apt-get upgrade -y
sudo apt-get install -y curl git apt-transport-https ca-certificates gnupg software-properties-common lsb-release

# 2. Installation de Docker
echo "--- 🐳 Installation de Docker ---"
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg

echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Ajout de l'utilisateur actuel au groupe docker
sudo usermod -aG docker $USER
echo "💡 Note : Tu devras te déconnecter/reconnecter pour utiliser Docker sans sudo."

# 3. Installation de Terraform
echo "--- 🏗️ Installation de Terraform ---"
curl -fsSL https://apt.releases.hashicorp.com/gpg | sudo gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/hashicorp.list
sudo apt-get update && sudo apt-get install -y terraform

# 4. Installation de kubectl
echo "--- ☸️ Installation de kubectl ---"
K8S_VERSION=$(curl -L -s https://dl.k8s.io/release/stable.txt)
curl -LO "https://dl.k8s.io/release/${K8S_VERSION}/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
rm kubectl

# 5. Installation de Kind
echo "--- 🎡 Installation de Kind ---"
curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.31.0/kind-linux-amd64
chmod +x ./kind
sudo mv ./kind /usr/local/bin/kind

echo "--- ✅ Configuration terminée avec succès ! ---"
echo "Utilise 'newgrp docker' pour activer les droits Docker immédiatement sans te reconnecter."
