use clap::{Parser,IntoApp};
use std::io::{self,BufRead};
use std::process::Stdio;
use regex::{Regex,Match};
use std::collections::{VecDeque,HashMap};
use std::cmp;
use ansi_term::Colour::{Black, Red, Green, Yellow, Blue, Purple, Cyan, Fixed};
use clap_complete::{generate, shells::Bash};
use std::iter;

#[derive(Parser, Debug)]
#[clap(author, 
       version, 
       about = "
       find groups of neighboring lines in text on STDIN, in which matches occur for all REGEXes from a given set
       ",
       long_about = None)]
struct Args {
    /// Generate bash completion
    #[clap(long)]
    completion: bool,    

    #[clap(short, long, value_name="RADIUS", default_value_t=8)]
    radius: usize,

    #[clap(short, long, value_name="MARGIN", default_value_t=4)]
    margin: usize,

    #[clap(long)]
    nocolor: bool,    

    #[clap(long)]
    nosep: bool,

    #[clap(long, value_name="SEPARATOR", default_value="━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")]
    sep: String,

    #[clap(long)]
    debug: bool,

    #[clap(value_name="REGEXEN")]
    regexen: Vec<String>

}

#[derive(Debug,Clone)]
struct MatchStarts(usize);
#[derive(Debug,Clone)]
struct MatchEnds(usize);
#[derive(Debug,Clone)]
struct PositionsOfMatch(Vec<(MatchStarts,MatchEnds)>);
impl PositionsOfMatch {
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}
#[derive(Debug)]
struct AnnotatedLine {
    text: String,
    matches: Vec<PositionsOfMatch>,
    n: i32
}
enum Marker { Norm, Marg, Rad, FinalLines }

fn matches_within_radius(regexen: &Vec<Regex>, buf: &VecDeque<AnnotatedLine>) -> bool {
    let mut all_terms_are_present = true;
    for i in 0..regexen.len() {
        if !(buf.iter().map(|al| al.matches[i].clone()).fold(false, |acc, m| acc || !(m.is_empty()))) { 
            all_terms_are_present = false 
        }
    }
    all_terms_are_present
}

fn show_line(al: AnnotatedLine, marker: Marker, lnum: i32, lines_to_print: usize) {
    let mut matches : Vec<(MatchStarts,MatchEnds)> = Vec::new();
    for v in al.matches { matches.extend(v.0); }
    let mut starts : Vec<usize> = matches.iter().map(|m| m.0.0).collect();
    let mut ends : Vec<usize> = matches.iter().map(|m| m.1.0).collect();
    starts.sort();
    ends.sort();
    let mut s : Vec<&usize> = starts.iter().filter(|i| { 
        starts.iter().filter(|j| j < i).collect::<Vec<&usize>>().len() == ends.iter().filter(|j| j <= i).collect::<Vec<&usize>>().len()
    }).collect();
    let mut e : Vec<&usize> = ends.iter().filter(|i| { 
        starts.iter().filter(|j| j < i).collect::<Vec<&usize>>().len() == ends.iter().filter(|j| j <= i).collect::<Vec<&usize>>().len()
    }).collect();
    s.dedup();
    e.dedup();
    match marker {
        Marker::Norm => (),
        Marker::Marg => print!("{} {}({})■", al.n, lnum,lines_to_print),
        Marker::Rad => print!("{} {}({})┃", al.n, lnum,lines_to_print),
        Marker::FinalLines => print!("{} {}({})◘", al.n, lnum,lines_to_print),
    }
    let mut prevend = 0;
    for i in s {
        let j = e.remove(0);
        print!("{}",&al.text[prevend..*i]);
        print!("{}",Green.bold().paint(&al.text[*i..*j]));
        prevend = *j;
    }
    println!("{}",&al.text[prevend..]);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let clops = Args::parse();

    if clops.completion {
        generate(Bash, &mut Args::command(), "plurigrep", &mut io::stdout()); 
        return Ok(())
    }

    let mark = |marker: Marker|  { if clops.debug { marker } else { Marker::Norm } };
    let regexen = clops.regexen.iter().map(|x| Regex::new(x).unwrap()).collect::<Vec<Regex>>();
    let mut rad_buf : VecDeque<AnnotatedLine> = VecDeque::new();
    let mut marg_buf : VecDeque<AnnotatedLine> = VecDeque::new();
    let mut lines_to_print = 0;
    let mut not_first = false;
    let mut line_no : i32 = -1;
    for line in io::stdin().lock().lines() {
        line_no = line_no + 1;
        let text = line.unwrap();
        let matches : Vec<PositionsOfMatch> = regexen
            .iter()
            .map(|r| PositionsOfMatch(
                    r.find_iter(&text)
                    .map(|m| (MatchStarts(m.start()),MatchEnds(m.end())))
                    .collect()
                    )
                )
            .collect();
        let annotated_line = AnnotatedLine { matches, text, n:line_no } ;
        // (A):
        rad_buf.push_back(annotated_line);
        // Obs: (lines_to_print > 0)  =>  (marg_buf is empty)
        // Indeed, the only place where marg_buf is pushded onto is (C) which only happens 
        // when rad_buf has max length; the only place where rad_buf is pushed onto is (A), 
        // but if lines_to_print > 0 then it is popped out in (B)
        if matches_within_radius(&regexen, &rad_buf) {
            if lines_to_print == 0 && !clops.nosep && not_first { println!("{}", clops.sep); }
            not_first = true;
            for l in marg_buf.drain(..) { show_line(l, mark(Marker::Marg), line_no, lines_to_print); }
            // before lines_to_print is increased, marg_buf is drained
            lines_to_print = rad_buf.len() + clops.margin ;
            if clops.debug { println!("___lines_to_print={}, rad_buffer_length={}_________",lines_to_print,rad_buf.len()); }
        } 
        if lines_to_print > 0 { 
            assert![marg_buf.is_empty()];
            // (B):
            match rad_buf.pop_front() {
                Some(l) => { show_line(l, mark(Marker::Rad), line_no, lines_to_print); lines_to_print = lines_to_print - 1; }
                None => ()
            }
        }
        // (C):
        if rad_buf.len() == clops.radius { rad_buf.pop_front().map(|l| marg_buf.push_back(l)); }
        if marg_buf.len() > clops.margin { marg_buf.pop_front(); }
    }
    for line in marg_buf.drain(..).chain(rad_buf.drain(..)) { 
        if lines_to_print > 0 { 
            show_line(line, mark(Marker::FinalLines), line_no, lines_to_print);
            lines_to_print = lines_to_print - 1;
        }
    }
    if clops.debug { println!("Remaining lines_to_print={}", lines_to_print); }
    Ok (())
}
