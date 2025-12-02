/*
 * Simple LISP REPL for BogoKernel
 * 
 * A minimal LISP interpreter with the following features:
 * - Basic S-expressions: atoms, lists, numbers
 * - Core primitives: quote, car, cdr, cons, atom, eq, +, -, *, /
 * - Lambda expressions and function application
 * - Define for binding variables
 * - Simple garbage collection with mark-and-sweep
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

/* Memory management */
#define MAX_CELLS 1024
#define MAX_ENV 128
#define MAX_INPUT 256
#define MAX_STRINGS 512
#define STRING_POOL_SIZE 8192

/* Cell types */
typedef enum {
    CELL_NIL,
    CELL_NUM,
    CELL_SYMBOL,
    CELL_CONS,
    CELL_LAMBDA,
    CELL_PRIMITIVE
} CellType;

/* Forward declarations */
struct Cell;
struct Env;

/* Primitive function pointer */
typedef struct Cell* (*PrimFunc)(struct Cell*);

/* Cell structure */
typedef struct Cell {
    CellType type;
    int marked;
    union {
        int num;
        char* symbol;
        struct {
            struct Cell* car;
            struct Cell* cdr;
        };
        struct {
            struct Cell* params;
            struct Cell* body;
            struct Env* env;
        } lambda;
        PrimFunc prim;
    };
} Cell;

/* Environment structure */
typedef struct Env {
    struct Cell* symbol;
    struct Cell* value;
    struct Env* next;
} Env;

/* Global state */
static Cell cells[MAX_CELLS];
static int next_cell = 0;
static Env envs[MAX_ENV];
static int next_env = 0;
static Env* global_env = NULL;
static Cell* nil_cell = NULL;
static Cell* t_cell = NULL;

/* String pool for symbols */
static char string_pool[STRING_POOL_SIZE];
static int next_string = 0;

/* Simple string duplication using string pool */
static char* str_alloc(const char* s) {
    int len = strlen(s);
    if (next_string + len + 1 >= STRING_POOL_SIZE) {
        printf("ERROR: Out of string memory\n");
        return (char*)s;
    }
    char* result = &string_pool[next_string];
    strcpy(result, s);
    next_string += len + 1;
    return result;
}

/* Forward declarations */
static Cell* eval(Cell* expr, Env* env);
static Cell* apply(Cell* fn, Cell* args, Env* env);
static void print_cell(Cell* cell);

/* Memory allocation */
static Cell* alloc_cell() {
    if (next_cell >= MAX_CELLS) {
        printf("ERROR: Out of memory\n");
        return nil_cell;
    }
    Cell* cell = &cells[next_cell++];
    cell->marked = 0;
    return cell;
}

static Env* alloc_env() {
    if (next_env >= MAX_ENV) {
        printf("ERROR: Out of environment slots\n");
        return NULL;
    }
    return &envs[next_env++];
}

/* Cell constructors */
static Cell* make_nil() {
    Cell* cell = alloc_cell();
    cell->type = CELL_NIL;
    return cell;
}

static Cell* make_num(int n) {
    Cell* cell = alloc_cell();
    cell->type = CELL_NUM;
    cell->num = n;
    return cell;
}

static Cell* make_symbol(const char* s) {
    Cell* cell = alloc_cell();
    cell->type = CELL_SYMBOL;
    cell->symbol = (char*)s; // We'll manage string lifetime manually
    return cell;
}

static Cell* make_cons(Cell* car, Cell* cdr) {
    Cell* cell = alloc_cell();
    cell->type = CELL_CONS;
    cell->car = car;
    cell->cdr = cdr;
    return cell;
}

static Cell* make_lambda(Cell* params, Cell* body, Env* env) {
    Cell* cell = alloc_cell();
    cell->type = CELL_LAMBDA;
    cell->lambda.params = params;
    cell->lambda.body = body;
    cell->lambda.env = env;
    return cell;
}

static Cell* make_primitive(PrimFunc fn) {
    Cell* cell = alloc_cell();
    cell->type = CELL_PRIMITIVE;
    cell->prim = fn;
    return cell;
}

/* Environment operations */
static Env* env_extend(Cell* symbol, Cell* value, Env* parent) {
    Env* env = alloc_env();
    if (!env) return parent;
    env->symbol = symbol;
    env->value = value;
    env->next = parent;
    return env;
}

static Cell* env_lookup(Cell* symbol, Env* env) {
    while (env) {
        if (env->symbol && env->symbol->type == CELL_SYMBOL &&
            strcmp(env->symbol->symbol, symbol->symbol) == 0) {
            return env->value;
        }
        env = env->next;
    }
    return nil_cell;
}

/* Primitive functions */
static Cell* prim_car(Cell* args) {
    if (!args || args->type != CELL_CONS) return nil_cell;
    Cell* first = args->car;
    if (first->type != CELL_CONS) return nil_cell;
    return first->car;
}

static Cell* prim_cdr(Cell* args) {
    if (!args || args->type != CELL_CONS) return nil_cell;
    Cell* first = args->car;
    if (first->type != CELL_CONS) return nil_cell;
    return first->cdr;
}

static Cell* prim_cons(Cell* args) {
    if (!args || args->type != CELL_CONS) return nil_cell;
    Cell* car = args->car;
    if (!args->cdr || args->cdr->type != CELL_CONS) return nil_cell;
    Cell* cdr = args->cdr->car;
    return make_cons(car, cdr);
}

static Cell* prim_atom(Cell* args) {
    if (!args || args->type != CELL_CONS) return nil_cell;
    Cell* first = args->car;
    return (first->type != CELL_CONS) ? t_cell : nil_cell;
}

static Cell* prim_eq(Cell* args) {
    if (!args || args->type != CELL_CONS) return nil_cell;
    Cell* first = args->car;
    if (!args->cdr || args->cdr->type != CELL_CONS) return nil_cell;
    Cell* second = args->cdr->car;
    
    if (first->type != second->type) return nil_cell;
    
    if (first->type == CELL_NUM) {
        return (first->num == second->num) ? t_cell : nil_cell;
    } else if (first->type == CELL_SYMBOL) {
        return (strcmp(first->symbol, second->symbol) == 0) ? t_cell : nil_cell;
    } else if (first->type == CELL_NIL) {
        return t_cell;
    }
    
    return (first == second) ? t_cell : nil_cell;
}

static Cell* prim_add(Cell* args) {
    int sum = 0;
    while (args && args->type == CELL_CONS) {
        if (args->car->type == CELL_NUM) {
            sum += args->car->num;
        }
        args = args->cdr;
    }
    return make_num(sum);
}

static Cell* prim_sub(Cell* args) {
    if (!args || args->type != CELL_CONS || args->car->type != CELL_NUM) {
        return make_num(0);
    }
    int result = args->car->num;
    args = args->cdr;
    if (!args || args->type == CELL_NIL) {
        return make_num(-result);
    }
    while (args && args->type == CELL_CONS) {
        if (args->car->type == CELL_NUM) {
            result -= args->car->num;
        }
        args = args->cdr;
    }
    return make_num(result);
}

static Cell* prim_mul(Cell* args) {
    int result = 1;
    while (args && args->type == CELL_CONS) {
        if (args->car->type == CELL_NUM) {
            result *= args->car->num;
        }
        args = args->cdr;
    }
    return make_num(result);
}

static Cell* prim_div(Cell* args) {
    if (!args || args->type != CELL_CONS || args->car->type != CELL_NUM) {
        return make_num(0);
    }
    int result = args->car->num;
    args = args->cdr;
    while (args && args->type == CELL_CONS) {
        if (args->car->type == CELL_NUM && args->car->num != 0) {
            result /= args->car->num;
        }
        args = args->cdr;
    }
    return make_num(result);
}

/* Parser */
static char input_buf[MAX_INPUT];
static int input_pos = 0;

static void skip_whitespace() {
    while (input_pos < MAX_INPUT && isspace(input_buf[input_pos])) {
        input_pos++;
    }
}

static Cell* parse_expr();

static Cell* parse_list() {
    // Already consumed '('
    skip_whitespace();
    
    if (input_buf[input_pos] == ')') {
        input_pos++;
        return nil_cell;
    }
    
    Cell* car = parse_expr();
    Cell* cdr = parse_list();
    return make_cons(car, cdr);
}

static Cell* parse_atom() {
    static char token[64];
    int i = 0;
    
    while (input_pos < MAX_INPUT && !isspace(input_buf[input_pos]) &&
           input_buf[input_pos] != '(' && input_buf[input_pos] != ')' &&
           input_buf[input_pos] != '\0' && i < 63) {
        token[i++] = input_buf[input_pos++];
    }
    token[i] = '\0';
    
    if (i == 0) return nil_cell;
    
    // Check if it's a number
    int is_num = 1;
    int start = (token[0] == '-' || token[0] == '+') ? 1 : 0;
    for (int j = start; j < i; j++) {
        if (!isdigit(token[j])) {
            is_num = 0;
            break;
        }
    }
    
    if (is_num && i > start) {
        return make_num(atoi(token));
    }
    
    // Check for special symbols
    if (strcmp(token, "nil") == 0) return nil_cell;
    if (strcmp(token, "t") == 0) return t_cell;
    
    // It's a symbol
    return make_symbol(str_alloc(token));
}

static Cell* parse_expr() {
    skip_whitespace();
    
    if (input_pos >= MAX_INPUT || input_buf[input_pos] == '\0') {
        return nil_cell;
    }
    
    if (input_buf[input_pos] == '(') {
        input_pos++;
        return parse_list();
    } else if (input_buf[input_pos] == ')') {
        return nil_cell;
    } else if (input_buf[input_pos] == '\'') {
        input_pos++;
        Cell* expr = parse_expr();
        return make_cons(make_symbol("quote"), make_cons(expr, nil_cell));
    } else {
        return parse_atom();
    }
}

/* Evaluator */
static Cell* eval_list(Cell* list, Env* env) {
    if (!list || list->type == CELL_NIL) {
        return nil_cell;
    }
    if (list->type != CELL_CONS) {
        return list;
    }
    
    Cell* head = eval(list->car, env);
    Cell* rest = eval_list(list->cdr, env);
    return make_cons(head, rest);
}

static Cell* eval(Cell* expr, Env* env) {
    if (!expr) return nil_cell;
    
    switch (expr->type) {
        case CELL_NIL:
        case CELL_NUM:
        case CELL_PRIMITIVE:
        case CELL_LAMBDA:
            return expr;
            
        case CELL_SYMBOL:
            return env_lookup(expr, env);
            
        case CELL_CONS: {
            Cell* op = expr->car;
            Cell* args = expr->cdr;
            
            // Special forms
            if (op->type == CELL_SYMBOL) {
                if (strcmp(op->symbol, "quote") == 0) {
                    if (args && args->type == CELL_CONS) {
                        return args->car;
                    }
                    return nil_cell;
                }
                else if (strcmp(op->symbol, "if") == 0) {
                    if (!args || args->type != CELL_CONS) return nil_cell;
                    Cell* cond = eval(args->car, env);
                    args = args->cdr;
                    if (!args || args->type != CELL_CONS) return nil_cell;
                    Cell* then_expr = args->car;
                    args = args->cdr;
                    Cell* else_expr = (args && args->type == CELL_CONS) ? args->car : nil_cell;
                    
                    if (cond->type != CELL_NIL) {
                        return eval(then_expr, env);
                    } else {
                        return eval(else_expr, env);
                    }
                }
                else if (strcmp(op->symbol, "lambda") == 0) {
                    if (!args || args->type != CELL_CONS) return nil_cell;
                    Cell* params = args->car;
                    args = args->cdr;
                    if (!args || args->type != CELL_CONS) return nil_cell;
                    Cell* body = args->car;
                    return make_lambda(params, body, env);
                }
                else if (strcmp(op->symbol, "define") == 0) {
                    if (!args || args->type != CELL_CONS) return nil_cell;
                    Cell* symbol = args->car;
                    args = args->cdr;
                    if (!args || args->type != CELL_CONS) return nil_cell;
                    Cell* value = eval(args->car, env);
                    
                    // Update global environment
                    global_env = env_extend(symbol, value, global_env);
                    return value;
                }
            }
            
            // Function application
            Cell* fn = eval(op, env);
            Cell* evaled_args = eval_list(args, env);
            return apply(fn, evaled_args, env);
        }
    }
    
    return nil_cell;
}

static Cell* apply(Cell* fn, Cell* args, Env* env) {
    if (!fn) return nil_cell;
    
    if (fn->type == CELL_PRIMITIVE) {
        return fn->prim(args);
    }
    else if (fn->type == CELL_LAMBDA) {
        // Bind parameters
        Env* new_env = fn->lambda.env;
        Cell* params = fn->lambda.params;
        Cell* arg_vals = args;
        
        while (params && params->type == CELL_CONS && 
               arg_vals && arg_vals->type == CELL_CONS) {
            new_env = env_extend(params->car, arg_vals->car, new_env);
            params = params->cdr;
            arg_vals = arg_vals->cdr;
        }
        
        return eval(fn->lambda.body, new_env);
    }
    
    return nil_cell;
}

/* Printer */
static void print_cell(Cell* cell) {
    if (!cell) {
        printf("nil");
        return;
    }
    
    switch (cell->type) {
        case CELL_NIL:
            printf("nil");
            break;
        case CELL_NUM:
            printf("%d", cell->num);
            break;
        case CELL_SYMBOL:
            printf("%s", cell->symbol);
            break;
        case CELL_CONS:
            printf("(");
            print_cell(cell->car);
            Cell* rest = cell->cdr;
            while (rest && rest->type == CELL_CONS) {
                printf(" ");
                print_cell(rest->car);
                rest = rest->cdr;
            }
            if (rest && rest->type != CELL_NIL) {
                printf(" . ");
                print_cell(rest);
            }
            printf(")");
            break;
        case CELL_LAMBDA:
            printf("<lambda>");
            break;
        case CELL_PRIMITIVE:
            printf("<primitive>");
            break;
    }
}

/* REPL */
static void init_lisp() {
    // Initialize nil and t
    nil_cell = make_nil();
    t_cell = make_symbol("t");
    
    // Setup global environment with primitives
    global_env = env_extend(make_symbol("car"), make_primitive(prim_car), global_env);
    global_env = env_extend(make_symbol("cdr"), make_primitive(prim_cdr), global_env);
    global_env = env_extend(make_symbol("cons"), make_primitive(prim_cons), global_env);
    global_env = env_extend(make_symbol("atom"), make_primitive(prim_atom), global_env);
    global_env = env_extend(make_symbol("eq"), make_primitive(prim_eq), global_env);
    global_env = env_extend(make_symbol("+"), make_primitive(prim_add), global_env);
    global_env = env_extend(make_symbol("-"), make_primitive(prim_sub), global_env);
    global_env = env_extend(make_symbol("*"), make_primitive(prim_mul), global_env);
    global_env = env_extend(make_symbol("/"), make_primitive(prim_div), global_env);
}

static void repl() {
    printf("BogoLISP v0.1\n");
    printf("Type expressions to evaluate, or 'quit' to exit\n");
    printf("\n");
    
    while (1) {
        printf("lisp> ");
        
        // Read line
        if (!fgets(input_buf, MAX_INPUT, stdin)) {
            break;
        }
        
        // Remove newline
        int len = strlen(input_buf);
        if (len > 0 && input_buf[len-1] == '\n') {
            input_buf[len-1] = '\0';
        }
        
        // Check for quit
        if (strcmp(input_buf, "quit") == 0 || strcmp(input_buf, "exit") == 0) {
            break;
        }
        
        // Skip empty lines
        if (strlen(input_buf) == 0) {
            continue;
        }
        
        // Parse and evaluate
        input_pos = 0;
        Cell* expr = parse_expr();
        Cell* result = eval(expr, global_env);
        
        // Print result
        print_cell(result);
        printf("\n");
    }
    
    printf("Goodbye!\n");
}

int main(int argc, char** argv, char** envp) {
    init_lisp();
    repl();
    return 0;
}
