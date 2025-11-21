#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define MAP_WIDTH 20
#define MAP_HEIGHT 10
#define MAX_ENEMIES 5
#define MAX_ITEMS 3

typedef struct {
    int x, y;
    int hp;
    int max_hp;
    int attack;
} Player;

typedef struct {
    int x, y;
    int hp;
    int attack;
    int alive;
} Enemy;

typedef struct {
    int x, y;
    int active;
} Item;

typedef struct {
    char tiles[MAP_HEIGHT][MAP_WIDTH];
    Player player;
    Enemy enemies[MAX_ENEMIES];
    Item items[MAX_ITEMS];
    int num_enemies;
    int num_items;
    int game_over;
    int won;
    char message[64];
} GameState;

// Simple pseudo-random number generator
static unsigned int seed = 12345;

int random_int(int max) {
    seed = seed * 1103515245 + 12345;
    return (seed / 65536) % max;
}

void clear_screen() {
    printf("\033[2J\033[H");
}

void init_game(GameState *game) {
    memset(game, 0, sizeof(GameState));
    
    // Initialize map with floors
    for (int y = 0; y < MAP_HEIGHT; y++) {
        for (int x = 0; x < MAP_WIDTH; x++) {
            if (y == 0 || y == MAP_HEIGHT - 1 || x == 0 || x == MAP_WIDTH - 1) {
                game->tiles[y][x] = '#';  // Walls
            } else {
                game->tiles[y][x] = '.';  // Floor
            }
        }
    }
    
    // Place player at random position
    game->player.x = 2 + random_int(5);
    game->player.y = 2 + random_int(3);
    game->player.hp = 20;
    game->player.max_hp = 20;
    game->player.attack = 5;
    
    // Place exit far from player
    int exit_x = MAP_WIDTH - 3 - random_int(3);
    int exit_y = MAP_HEIGHT - 3 - random_int(2);
    game->tiles[exit_y][exit_x] = 'X';
    
    // Spawn enemies
    game->num_enemies = 3 + random_int(3);
    for (int i = 0; i < game->num_enemies; i++) {
        int x, y;
        do {
            x = 1 + random_int(MAP_WIDTH - 2);
            y = 1 + random_int(MAP_HEIGHT - 2);
        } while ((x == game->player.x && y == game->player.y) || 
                 game->tiles[y][x] != '.');
        
        game->enemies[i].x = x;
        game->enemies[i].y = y;
        game->enemies[i].hp = 10;
        game->enemies[i].attack = 3;
        game->enemies[i].alive = 1;
    }
    
    // Spawn health potions
    game->num_items = 2 + random_int(2);
    for (int i = 0; i < game->num_items; i++) {
        int x, y;
        do {
            x = 1 + random_int(MAP_WIDTH - 2);
            y = 1 + random_int(MAP_HEIGHT - 2);
        } while ((x == game->player.x && y == game->player.y) || 
                 game->tiles[y][x] != '.');
        
        game->items[i].x = x;
        game->items[i].y = y;
        game->items[i].active = 1;
    }
    
    strcpy(game->message, "Welcome to CRogue! WASD to move, Q to quit.");
}

void render(GameState *game) {
    clear_screen();
    
    // Draw map
    for (int y = 0; y < MAP_HEIGHT; y++) {
        for (int x = 0; x < MAP_WIDTH; x++) {
            char ch = game->tiles[y][x];
            
            // Check for player
            if (x == game->player.x && y == game->player.y) {
                ch = '@';
            } else {
                // Check for enemies
                for (int i = 0; i < game->num_enemies; i++) {
                    if (game->enemies[i].alive && 
                        x == game->enemies[i].x && y == game->enemies[i].y) {
                        ch = 'E';
                        break;
                    }
                }
                
                // Check for items
                for (int i = 0; i < game->num_items; i++) {
                    if (game->items[i].active && 
                        x == game->items[i].x && y == game->items[i].y) {
                        ch = 'H';
                        break;
                    }
                }
            }
            
            putchar(ch);
        }
        putchar('\n');
    }
    
    // Draw stats
    printf("\nHP: %d/%d  Attack: %d  Enemies: %d\n", 
           game->player.hp, game->player.max_hp, game->player.attack,
           game->num_enemies);
    printf("%s\n", game->message);
    printf("\n[W/A/S/D] Move  [Q] Quit\n");
}

char read_input() {
    char c;
    read(0, &c, 1);
    return c;
}

void combat(GameState *game, Enemy *enemy) {
    // Player attacks enemy
    enemy->hp -= game->player.attack;
    
    if (enemy->hp <= 0) {
        enemy->alive = 0;
        game->num_enemies--;
        strcpy(game->message, "You defeated the enemy!");
        return;
    }
    
    // Enemy attacks player
    game->player.hp -= enemy->attack;
    
    if (game->player.hp <= 0) {
        game->game_over = 1;
        strcpy(game->message, "You died!");
    } else {
        sprintf(game->message, "Combat! Enemy HP: %d, Your HP: %d", 
                enemy->hp, game->player.hp);
    }
}

void process_input(GameState *game, char cmd) {
    int new_x = game->player.x;
    int new_y = game->player.y;
    
    switch (cmd) {
        case 'w': case 'W': new_y--; break;
        case 's': case 'S': new_y++; break;
        case 'a': case 'A': new_x--; break;
        case 'd': case 'D': new_x++; break;
        case 'q': case 'Q':
            game->game_over = 1;
            strcpy(game->message, "Thanks for playing!");
            return;
        default:
            strcpy(game->message, "Use WASD to move, Q to quit.");
            return;
    }
    
    // Check bounds
    if (new_x < 0 || new_x >= MAP_WIDTH || new_y < 0 || new_y >= MAP_HEIGHT) {
        return;
    }
    
    // Check walls
    if (game->tiles[new_y][new_x] == '#') {
        strcpy(game->message, "You bump into a wall.");
        return;
    }
    
    // Check exit
    if (game->tiles[new_y][new_x] == 'X') {
        game->game_over = 1;
        game->won = 1;
        strcpy(game->message, "You found the exit! You win!");
        return;
    }
    
    // Check for enemies
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (game->enemies[i].alive && 
            new_x == game->enemies[i].x && new_y == game->enemies[i].y) {
            combat(game, &game->enemies[i]);
            return;
        }
    }
    
    // Check for items
    for (int i = 0; i < game->num_items; i++) {
        if (game->items[i].active && 
            new_x == game->items[i].x && new_y == game->items[i].y) {
            game->items[i].active = 0;
            game->player.hp += 10;
            if (game->player.hp > game->player.max_hp) {
                game->player.hp = game->player.max_hp;
            }
            strcpy(game->message, "You found a health potion! +10 HP");
            game->player.x = new_x;
            game->player.y = new_y;
            return;
        }
    }
    
    // Move player
    game->player.x = new_x;
    game->player.y = new_y;
    strcpy(game->message, "");
}

void show_game_over(GameState *game) {
    clear_screen();
    printf("\n");
    printf("  +==============================+\n");
    printf("  |       GAME OVER              |\n");
    printf("  +==============================+\n");
    printf("\n");
    
    if (game->won) {
        printf("  *** VICTORY! ***\n");
        printf("  You escaped the dungeon!\n");
    } else if (game->player.hp <= 0) {
        printf("  *** DEFEAT ***\n");
        printf("  You were slain in the dungeon.\n");
    } else {
        printf("  Thanks for playing!\n");
    }
    
    printf("\n  Final Stats:\n");
    printf("  HP: %d/%d\n", game->player.hp, game->player.max_hp);
    printf("  Enemies Defeated: %d\n", MAX_ENEMIES - game->num_enemies);
    printf("\n");
}

int main(int argc, char **argv, char **envp) {
    GameState game;
    
    // Use time-based seed if available
    seed = 12345;
    
    init_game(&game);
    
    while (!game.game_over) {
        render(&game);
        char cmd = read_input();
        process_input(&game, cmd);
    }
    
    render(&game);
    show_game_over(&game);
    
    return 0;
}
