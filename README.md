# Instructions pour le lab 
Le script (`setup-dev-env.sh`) d'installation et de configuration des softs est dans le répertoire `./kube/`. Il nécessite d'être root ou sudoer.

# Déroulement
1. Construire l'application  
Autant compiler directement dans un conteneur pour ensuite copier le binaire dans une image prête à l'emploi. C'est du _multistage build_ et cela permet de profiter d'un environnement de compilation conteneurisé (donc sans impact sur notre machine hôte) sans pour autant alourdir l'image finale.  

`docker build --tag lab-api:v2 .`  

- `--tag lab-api:v2` sert à identifier l'image créée avec le même nom utilisé dans la suite du lab.  
- `.` indique à l'instruction `build` de travailler à partir du répertoire courant, donc d'y chercher Dockerfile et fichiers à copier.  

2. Tester l'application en local   
`docker run --rm --name lab-api --publish 8080:8080 --env RUST_LOG=debug lab-api:v2`  
- `--rm` demande à docker de supprimer le conteneur dès son arrêt, car par défaut il le conserverait tout de même sur le disque.  
- `--name lab-api` permet de nommer le conteneur qui tourne, plutôt que de se contenter du nom par défaut qui lui serait affecté.  
- `--publish 8080:8080` indique de publier le port 8080 du conteneur sur le port local 8080 de la machine hôte, afin que l'application soit accessible depuis `localhost:8080`.  
- `--env RUST_LOG=debug` indique de passer au conteneur dans une variable d'environnement `RUST_LOG` la valeur `debug` pour pouvoir lire des logs applicatives détaillées sur la sortie standard.  
  
`curl -v http://localhost:8080/hello` --> OK http 200  
`curl -v http://localhost:8080/items` --> KO http 500  
Elle tourne, mais en raison de l'abscence d'une base de données, une partie des fonctionnalités n'est pas disponible.  
On peut s'en débarrasser pour le moment.  
`docker stop lab-api`  
  
3. Déployer et exposer l'application dans kube  

Préalable : Pousser l'application dans la registry afin qu'elle soit déployable de partout d'où cette regitry est accessible (non, il n'y a pas d'intérêt en localhost, mais c'est bien le but d'un lab...)  
`docker tag lab-api:v2 localhost:5001/lab-api:v2`  
On taggue l'image avec l'url de la registry pour pouvoir l'y pousser.  
`docker push localhost:5001/lab-api:v2`  
Et voilà, maintenant l'image est récupérable avec un `docker pull localhost:5001/lab-api:v2`, et en particulier du cluster kube.  

On peut créer le déploiement :  
`kubectl apply -f api-deployment.yml`  
Vérifier que les objets kube sont bien créés et noter leur état.  
`kubectl get pods`  
`kubectl get deployments`  
`kubectl get services`  
  
4. Constater que cette fois l'application n'est pas accessible : kube ne la considère pas comme `READY`, donc ne lui envoie pas de flux.  

`curl -v http://localhost:8080/hello` --> KO Connection refused  
Pour autant, il ne l'éteint pas, ni ne la redémarre. C'est la différence (et l'intérêt) entre la `livenessProbe`, qui pilote le redémarrage du pod et la `readynessProbe`, qui elle, permet à l'application d'indiquer qu'elle est en état de recevoir et de traiter du flux.   
  
Ici la `livenessProbe` interroge le endpoint `/health/live` qui ne peut échouer que si l'application n'est pas démarrée (ce qui le rend intéressant pour cet usage).  
  
La `readynessProbe` interroge le endpoint `/health/ready`, qui lui par contre, ne renvoie une réponse ok que si l'application est démarrée *et* que la base accepte des requêtes SQL. En résumé, que si l'application est effectivement prête à un fonctionnement nominal.  
  
5. Sur le même modèle que l'application, déployer la base de données ainsi que son script d'initialisation (création de la table et peuplement de deux objets).  
`kubectl apply -f postgres-configmap.yml`  
`kubectl apply -f postgres-deployment.yml`  
`kubectl get pods`  
`kubectl get deployments`  
`kubectl get services`  
  
6. Incroyable, cela a suffit à remettre la prestation en état ! Sans avoir eu besoin de la redémarrer ou d'altérer son déploiement.   
`curl -v http://localhost:8080/hello` --> OK  
`curl -v http://localhost:8080/items` --> OK  
`curl -v -XPOST http://localhost:8080/hello` --> OK  
  
 C'est le propre de kubernetes : le système fait en sorte de _converger_ vers l'état désiré, décrit dans les objets de déploiement, plutôt que d'exécuter des actions prévues à l'avance dans un ordre défini.  
  
7. Pour illustrer les capacités de mise á l'échelle horizontale automatique, déployer un objet `HorizontalPodAutoscaler` qui va changer le nombre de pods du d2ploiement en fonction de métriques émises par celui-ci.  
`kubectl apply -f api-autoscaler.yml`  
  
8. Injecter de la charge et attendre un peu.  
`while true; do curl http://localhost:8080/hello; curl http://localhost:8080/hello; curl http://localhost:8080/items; curl http://localhost:8080/items; done;`  
  
9. Constater que de nouveaux pods sont déployés en réaction. 🪄  
`watch kubectl get pods`   
  
10. Supprimer tous les objets à l'aide de `kubectl delete` et constater que ce mode opératoire ne laisse pas traîner de ressources : le ménage est bien fait.  
`kubectl delete -f api-autoscaler.yml -f api-deployment.yml -f postgres-configmap.yml -f postgres-deployment.yml`  
`kubectl get all`  
  
(la présence du service `service/kubernetes` est normale, il vient avec le cluster)
