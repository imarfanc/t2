# shell-2.3

# install nvm, vpm, go, deno, miniconda, oh my zsh

## miniconda

[/miniconda/install#quickstart-install-instructions](https://www.anaconda.com/docs/getting-started/miniconda/install#quickstart-install-instructions)

### 1

```bash
mkdir -p ~/miniconda3
curl https://repo.anaconda.com/miniconda/Miniconda3-latest-MacOSX-arm64.sh -o ~/miniconda3/miniconda.sh
bash ~/miniconda3/miniconda.sh -b -u -p ~/miniconda3
rm ~/miniconda3/miniconda.sh
```

### 2

```bash
source ~/miniconda3/bin/activate
```

### 3

```bash
conda init --all
```

## check installs from previous step

```bash
brew --version
conda --version
python --version
pip --version
perl -v                     # no need to update to latest
```

---

## oh my zsh

### step 1

[https://ohmyz.sh/](https://ohmyz.sh/)

```bash
sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
```

### step 2

nothing

### step 3

---

## uv

### step 1

[https://docs.astral.sh/uv/](https://docs.astral.sh/uv/)

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### step 2

```bash
# add to home
source $HOME/.local/bin/env
```

---

## nvm & npm & node

### step 1

[https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating](https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating)

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
```

### step 2

```bash
# restart terminal OR
source ~/.zshrc
nvm install node
```

---

## deno

### step 1

[https://docs.deno.com/runtime/getting_started/installation/](https://docs.deno.com/runtime/getting_started/installation/)

```bash
curl -fsSL https://deno.land/install.sh | sh
```

### step 2

```bash
# follow steps
# restart terminal OR
source ~/.zshrc
```

---

## vt cli

### step 1

[https://github.com/val-town/vt](https://github.com/val-town/vt)

```bash
deno install -grAf jsr:@valtown/vt
```

### step 2

[https://www.val.town/x/arfan/my-Vals](https://www.val.town/x/arfan/my-Vals)

```bash
mkdir -p ~/Developer/local-dev-Apps/vt-cli-arfan
cd ~/Developer/local-dev-Apps/vt-cli-arfan
vt clone https://www.val.town/x/arfan/fast-md
cd fast-md
vt status
cursor .
```

---

```bash
omz version
uv --version
nvm -v
npm -v
node -v
deno --version
```
