use libkvbaza::BazaKV;

#[cfg(target_os = "windows")]
const KORISCENJE: &str = "
KORISCENJE:
    kv_mem.exe FAJL uzmi KLJUC
    kv_mem.exe FAJL obrisi KLJUC
    kv_mem.exe FAJL unesi KLJUC VREDNOST
    kv_mem.exe FAJL azuriraj KLJUC VREDNOST
";

#[cfg(not(target_os = "windows"))]
const KORISCENJE: &str = "
KORISCENJE:
    kv_mem FAJL uzmi KLJUC
    kv_mem FAJL obrisi KLJUC
    kv_mem FAJL unesi KLJUC VREDNOST
    kv_mem FAJL azuriraj KLJUC VREDNOST
";

fn main() {
    let argumenti: Vec<String> = std::env::args().collect();
    let ime_fajla = argumenti.get(1).expect(&KORISCENJE);
    let radnja = argumenti.get(2).expect(&KORISCENJE).as_ref();
    let kljuc = argumenti.get(3).expect(&KORISCENJE).as_ref();
    let mozda_vrednost = argumenti.get(4);
  
    let putanja = std::path::Path::new(&ime_fajla);
    let mut a = BazaKV::otvori(putanja).expect("neuspelo otvaranje fajla");
    a.ucitaj().expect("neuspelo ucitavanje podataka");
  
    match radnja {
      "uzmi" => {
        match a.uzmi(kljuc).unwrap() {
          None => eprintln!("{:?} nije pronadjen", kljuc),
          Some(vrednost) => println!("{:?}", vrednost),
        }
      }
      "obrisi" => a.obrisi(kljuc).unwrap(),
      "unesi" => {
        let vrednost = mozda_vrednost.expect(&KORISCENJE).as_ref();
        a.unesi(kljuc, vrednost).unwrap()
      }
      "azuriraj" => {
        let vrednost = mozda_vrednost.expect(&KORISCENJE).as_ref();
        a.azuriraj(kljuc, vrednost).unwrap()
      }
      _ => eprintln!("{}", &KORISCENJE),
    }
  }