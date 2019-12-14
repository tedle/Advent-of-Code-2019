use std::collections::HashMap;

#[derive(Debug)]
struct Recipe {
    inputs: Vec<Ingredient>,
    output: Ingredient,
}
impl Recipe {
    fn new(inputs: Vec<Ingredient>, output: Ingredient) -> Recipe {
        Recipe { inputs, output }
    }
}
type RecipeBook = HashMap<String, Recipe>;
type Ingredient = (usize, String);

fn parse_ingredient(ingredient: &str) -> Ingredient {
    let measurement = ingredient.split(" ").collect::<Vec<&str>>();
    let quantity = measurement[0].parse::<usize>().unwrap();
    let material = String::from(measurement[1]);
    (quantity, material)
}

fn parse_input(filename: &str) -> RecipeBook {
    let input = std::fs::read_to_string(filename).unwrap();
    let mut recipes = RecipeBook::new();
    for line in input.lines() {
        let sides = line.split("=>").collect::<Vec<&str>>();
        let raw_ingredients = sides[0].split(",").map(|i| i.trim()).collect::<Vec<&str>>();
        let ingredients = raw_ingredients
            .iter()
            .map(|i| parse_ingredient(i))
            .collect::<Vec<Ingredient>>();
        let result = parse_ingredient(sides[1].trim());
        recipes.insert(result.1.clone(), Recipe::new(ingredients, result));
    }
    recipes
}

fn ore_cost(
    ingredient: &Ingredient,
    inventory: &mut HashMap<String, usize>,
    recipes: &RecipeBook,
) -> usize {
    let mut ore = 0;
    let recipe = recipes.get(ingredient.1.as_str()).unwrap();
    let multi = (ingredient.0 as f64 / recipe.output.0 as f64).ceil() as usize;
    for i in &recipe.inputs {
        let mut needed = i.0 * multi;
        let premade = inventory.entry(i.1.clone()).or_insert(0);
        if needed < *premade {
            *premade -= needed;
            continue;
        } else {
            needed -= *premade;
            *premade = 0;
        }
        ore += if i.1 == "ORE" {
            needed
        } else {
            ore_cost(&(needed, i.1.clone()), inventory, recipes)
        };
    }
    let excess = recipe.output.0 * multi - ingredient.0;
    let output_inv = inventory.entry(ingredient.1.clone()).or_insert(0);
    *output_inv += excess;
    ore
}

fn search<T>(limit: usize, function: T) -> usize
where
    T: Fn(usize) -> usize,
{
    let mut higher = 1;
    while function(higher) < limit {
        higher *= 2;
    }
    fn recursive_search<T>(lower: usize, higher: usize, limit: usize, function: T) -> usize
    where
        T: Fn(usize) -> usize,
    {
        if lower == higher - 1 {
            return lower;
        }
        let n = (lower + higher) / 2;
        let result = function(n);
        if result > limit {
            recursive_search(lower, n, limit, function)
        } else if result < limit {
            recursive_search(n, higher, limit, function)
        } else {
            n
        }
    };
    recursive_search(1, higher, limit, function)
}

fn main() {
    let recipes = parse_input("input");
    println!(
        "14-1:\n{}",
        ore_cost(&(1, String::from("FUEL")), &mut HashMap::new(), &recipes)
    );
    println!(
        "14-2:\n{}",
        search(1_000_000_000_000, |fuel| {
            ore_cost(&(fuel, String::from("FUEL")), &mut HashMap::new(), &recipes)
        })
    );
}
