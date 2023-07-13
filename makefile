# コンパイラとコンパイルオプション
CC = gcc
CFLAGS = -Wall -Wextra

# ターゲットと依存関係、コンパイルコマンド
TARGET = my_program
SRCS = main.c foo.c bar.c
OBJS = $(SRCS:.c=.o)

run: $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^

build:
	cargo build

clean:
	cargo clean
