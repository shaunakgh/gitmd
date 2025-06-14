============ gitmd ============
 An ai-powered README/blog/writeup 
 generator for git projects 
 made with rust 

============ setup ============
Prerequisites:
==============
If you haven't already, install rust:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
and restart your terminal session.

Then, install the ollama application and cli
at https://ollama.com/download

Since this program runs llama3.2 ai by default, run this command:
ollama run llama3.2
once finished installing the model, exit with ctrl-d
NOTE that this model will be 2.0gb

Installing
==========
Run this command in your terminal session:
git clone https://github.com/shaunakgh/gitmd.git && cd gitmd && cargo install --path .
Then, run this command to see if it has installed
successfully:
gitmd
If it returns this:
error: the following required arguments were not provided:
  <--readme|--blog|--writeup>

Usage: gitmd <--readme|--blog|--writeup>

For more information, try '--help'.
Congrats 🎉! Gitmd is installed on your computer.

============ usage ============
gitmd -p PATH -m MODEL -r/-b/-w
where:
OPTIONAL
-p or --path is the absolute path to your git repo. 
DEFAULTS TO current directory
-m or --model is the model
DEFAULTS TO llama3.2
MANDATORY
-r/-b/-w for README, blog or writeup
