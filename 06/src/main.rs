use std::collections::HashMap;

#[derive(Debug)]
struct Orbit {
    parent: String,
    child: String,
}

fn parse_input(filename: &str) -> Vec<Orbit> {
    let input = std::fs::read_to_string(filename).unwrap();
    input
        .lines()
        .map(|line| {
            let values = line.split(")").map(String::from).collect::<Vec<String>>();
            Orbit {
                parent: values[0].clone(),
                child: values[1].clone(),
            }
        })
        .collect()
}

#[derive(Default)]
struct Planet {
    parent: Option<String>,
    children: Vec<String>,
    orbit_depth: usize,
}

struct PlanetMap(HashMap<String, Planet>);

impl std::ops::Deref for PlanetMap {
    type Target = HashMap<String, Planet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PlanetMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&str> for PlanetMap {
    fn from(filename: &str) -> PlanetMap {
        let mut map = PlanetMap(HashMap::new());
        for orbit in parse_input(filename) {
            let child = map.entry(orbit.child.to_string()).or_default();
            child.parent = Some(orbit.parent.to_string());
            let planet = map.entry(orbit.parent.to_string()).or_default();
            planet.children.push(orbit.child);
        }

        map.calculate_orbit_depth("COM");
        map
    }
}

impl PlanetMap {
    fn calculate_orbit_depth(&mut self, planet: &str) {
        fn add(planet: &str, map: &mut PlanetMap, depth: usize) {
            let children: Vec<String> = map.get(planet).unwrap().children.to_vec();
            for planet_name in &children {
                let child = map.get_mut(planet_name).unwrap();
                child.orbit_depth += depth;
                add(planet_name, map, depth + 1);
            }
        };
        add(planet, self, 1);
    }

    fn find_total_orbits(&self) -> usize {
        self.values()
            .fold(0, |acc, planet| acc + planet.orbit_depth)
    }

    fn find_closest_path(&self, a: &str, b: &str) -> usize {
        let mut a_search = self.get(a).unwrap();
        let mut b_search = self.get(b).unwrap();
        let a_depth = a_search.orbit_depth;
        let b_depth = b_search.orbit_depth;

        while a_search.parent != b_search.parent {
            if a_search.orbit_depth > b_search.orbit_depth {
                a_search = self.get(a_search.parent.as_ref().unwrap()).unwrap();
            } else {
                b_search = self.get(b_search.parent.as_ref().unwrap()).unwrap();
            }
        }

        (a_depth - a_search.orbit_depth) + (b_depth - b_search.orbit_depth)
    }
}

fn main() {
    let map = PlanetMap::from("input");

    println!("6-1:\n{}", map.find_total_orbits());
    println!("6-2:\n{}", map.find_closest_path("YOU", "SAN"));
}
