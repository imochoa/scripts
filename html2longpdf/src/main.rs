use quicli::prelude::*;
use structopt::StructOpt;
// use std::env;
use std::fs::remove_file;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
// use uuid::Uuid;

/// Make sure the executables are there before calling it
fn get_path_to_cmd(cmd: &str) -> PathBuf {
    let output = Command::new("which")
        .arg(cmd)
        .output()
        .expect("Failed to find program!");
    assert!(output.status.success());
    let p = PathBuf::from(
        String::from_utf8(output.stdout)
            .expect("Could not parse output")
            .trim(),
    );
    assert!(p.try_exists().is_ok());
    return p;
}

#[derive(Debug, PartialEq)]
enum PDFUnit {
    Px,
    Mm,
    Inch,
}
impl PDFUnit {
    fn from_str(s: &str) -> PDFUnit {
        match s {
            "mm" => PDFUnit::Mm,
            "in" => PDFUnit::Inch,
            "px" | "pts" => PDFUnit::Px,
            _ => panic!("No match!"),
        }
    }
    fn to_string(&self) -> String {
        match self {
            PDFUnit::Mm => "mm",
            PDFUnit::Inch => "in",
            PDFUnit::Px => "px",
        }
        .to_string()
    }
}

/// Results from running the pdfinfo cmd
#[derive(Debug)]
struct PDFInfo {
    pages: u8,
    width: u16,
    height: u16,
    /// pts, mm
    unit: PDFUnit,
}

impl PDFInfo {
    /// Includes conversion when using output.stdout
    // pub fn from_stdout(stdout: &Vec<u8>) -> Self {
    //     PDFInfo::from_stdout_str(
    //         String::from_utf8(stdout)
    //             .expect("Could not parse output")
    //             .trim(),
    //     )
    // }

    /// Includes conversion when using output.stdout
    pub fn from_stdout(stdout: Vec<u8>) -> Self {
        let mut pages = None;
        // let mut pagesize = None;
        let mut width = None;
        let mut height = None;
        let mut unit = None;

        for line in String::from_utf8(stdout)
            .expect("err")
            .split("\n")
            .into_iter()
        {
            let mut line_iter = line.split(":");
            let key = line_iter.next().unwrap();
            // println!("{:#?}", &key);
            match key.to_lowercase().as_str() {
                "pages" => pages = Some(line_iter.next().unwrap().trim().parse::<u8>().unwrap()),
                "page size" => {
                    // let value = line_iter.next().unwrap();

                    // let pagesize_str = line_iter.next().unwrap().trim().to_string();
                    // let vv: Vec<_> = value.split_whitespace().collect();
                    let vv: Vec<_> = line_iter.next().unwrap().split_whitespace().collect();
                    // let (h, _, w, u, _) = value.split_whitespace().collect();

                    // println!("{:#?}", vv);
                    // println!("{:#?}", vv.get(0));

                    // line_iter.width = Some(0);
                    width = Some(vv.get(0).unwrap().parse::<u16>().unwrap());
                    height = Some(vv.get(2).unwrap().parse::<u16>().unwrap());
                    unit = Some(PDFUnit::from_str(vv.get(3).unwrap()));
                    // pagesize = Some(.to_string());
                }
                _ => continue,
            }
            // println!("{:#?}", line.split(":").next().unwrap());
        }
        PDFInfo {
            pages: pages.unwrap(), //pages.ok_or(""),
            // pagesize: pagesize.unwrap(), //pages.ok_or(""),
            width: width.unwrap(),   //pages.ok_or(""),
            height: height.unwrap(), //pages.ok_or(""),
            unit: unit.unwrap(),     //pages.ok_or(""),
        }
    }
}

/// Merge all pages
fn html_to_1page_pdf(html: &Path, pdf: &Path) {
    let wkhtmltopdf_cmd = get_path_to_cmd("wkhtmltopdf");
    let pdfinfo_cmd = get_path_to_cmd("pdfinfo");

    println!("{:#?}\n{:#?}", &html, &pdf);
    println!("{:#?}\n{:#?}", &wkhtmltopdf_cmd, &pdfinfo_cmd);

    // let tempdir = env::temp_dir();
    // println!("{:#?}", &tempdir);
    // let uuid = Uuid::new_v4();
    // println!("{:#?}", &uuid);
    // Find required pages
    // let output = Command::new(pdfinfo_cmd).arg(pdf)
    //     .output()
    //     .expect("Failed to find program!");
    //
    // let pdfinfo = PDFInfo::from_stdout()

    println!("Generate long PDF");
    if remove_file(pdf).is_ok() {
        println!("Deleting...");
    }
    // "--margin-top", "0", "--margin-bottom", "0"
    // --page-width "${W}mm" --page-height "${H}mm"
    let wk1 = Command::new(&wkhtmltopdf_cmd)
        .args([&html, &pdf])
        .output()
        .expect("Failed to run!");
    assert!(wk1.status.success());

    println!("Find number of pages");
    let pi1 = Command::new(&pdfinfo_cmd)
        .arg(&pdf)
        .output()
        .expect("Failed to run!");
    assert!(pi1.status.success());
    let pdfinfo1 = PDFInfo::from_stdout(pi1.stdout);
    println!("{:#?}", &pdfinfo1);
    remove_file(pdf).expect("no PDF generated!");
    // px is valid (pts is not!)
    // mm
    // in
    // let u = "px";
    let u = pdfinfo1.unit.to_string();
    let pagewidth = format!("{}{}", pdfinfo1.width, u);

    let pageheight = format!("{}{}", (pdfinfo1.pages as u16) * pdfinfo1.height, u);

    // let pageheight = format!("{}", (pdfinfo1.pages as u16) * pdfinfo1.height);
    // let pageheight = "222222mm".to_string();
    println!("{:#?} mm", &pageheight);

    let wk2 = Command::new(&wkhtmltopdf_cmd)
        .args([
            // "-T",
            // "0mm",
            // "-B",
            // "0mm",
            "--page-height",
            pageheight.as_str(),
            "--page-width",
            pagewidth.as_str(),
            html.to_str().unwrap(),
            pdf.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run!");
    println!("{:#?}", String::from_utf8(wk2.stderr).unwrap());

    assert!(wk2.status.success());
    //
    //
    //
    // 	//    # wkhtmltopdf -T 0 -B 0 --page-width "${W}mm" --page-height "${H}mm" "$1" "${AUX_PDF}"
    // 	//
    // 	//    # -B, --margin-bottom <unitreal>      Set the page bottom margin
    // 	//    # -L, --margin-left <unitreal>        Set the page left margin (default 10mm)
    // 	//    # -R, --margin-right <unitreal>       Set the page right margin (default 10mm)
    // 	//    # -T, --margin-top <unitreal>         Set the page top margin
    // 	//
    // 	//    # -d, --dpi <dpi>                     Change the dpi explicitly (this has no
    // 	//    #                                     effect on X11 based systems) (default 96)
    // 	//
    // 	//    # --background                        Do print background (default)
    //
    // 	// Initial estimate
    // 	// tmpPath := filepath.Join(os.TempDir(), fmt.Sprintf("%d.pdf", rand.Float64()))
    // 	// TODO get a good random path!
    // 	tmpPath := filepath.Join(os.TempDir(), "temppath.pdf")
    //
    // 	// How many pages of (W,H) are required?
    // 	_, _ = exec.Command("wkhtmltopdf",
    // 		"-T", "0", "-B", "0",
    // 		"--page-width", fmt.Sprintf("%s%s", strconv.Itoa(W), units),
    // 		"--page-height", fmt.Sprintf("%s%s", strconv.Itoa(H), units),
    // 		inHTML, tmpPath,
    // 	).Output()
    //
    // 	pageCount := getPdfPageCount(tmpPath)
    //
    // 	// Make long PDF
    // 	_, _ = exec.Command("wkhtmltopdf",
    // 		"-T", "0", "-B", "0",
    // 		"--page-width", fmt.Sprintf("%s%s", strconv.Itoa(W), units),
    // 		"--page-height", fmt.Sprintf("%s%s", strconv.Itoa(H*pageCount), units),
    // 		inHTML, outPDF,
    // 	).Output()
    //

    // 	log.Printf("Found [wkhtmltopdf] (Version: %s)", wkhtmltopdfVersion())
    // 	log.Printf("Found [pdfinfo] (Version: %s)", pdfinfoVersion())

    println!("Done!");
}

/// Read some lines of a file
#[derive(Debug, StructOpt)]
struct Cli {
    /// Input file to read
    input_html: String,
    /// Output PDF
    output_pdf: String,
    // Number of lines to read
    // #[structopt(short = "n")]
    // num: usize,
}

fn main() -> CliResult {
    // let input_html = Path::new("/home/imo/Code/scripts/html2longpdf/file.html");
    // let output_pdf = Path::new("/home/imo/Code/scripts/html2longpdf/out.pdf");

    let args = Cli::from_args();
    // read_file(&args.file)?
    //     .lines()
    //     .take(args.num)
    //     .for_each(|line| println!("{}", line));
    // Ok(())

    html_to_1page_pdf(&Path::new(&args.input_html), &Path::new(&args.output_pdf));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn pdfinfo_stdout_parsing() {
        let stdout = "
Title:           aaaa bbbb cccccccc dddddd eeeee (Version 3)
Subject:         This guide describes how to use the aaaa aaaa aaaaaaaa, a software tool
Keywords:        ; \"data exchange, interoperability\"
Author:          AAAAAA, BBBBBB A. B. ;
Creator:         Acrobat PDFMaker 11 for Word
Producer:        Adobe PDF Library 11.0
CreationDate:    Wed Apr 27 17:13:30 2016 CEST
ModDate:         Thu Apr  7 15:50:45 2022 CEST
Custom Metadata: yes
Metadata Stream: yes
Tagged:          yes
UserProperties:  no
Suspects:        no
Form:            AcroForm
JavaScript:      no
Pages:           59
Encrypted:       no
Page size:       612 x 792 pts (letter)
Page rot:        0
File size:       2096853 bytes
Optimized:       no
PDF version:     1.5
"
        .as_bytes()
        .to_vec();

        let info = PDFInfo::from_stdout(stdout);
        println!("{:#?}", info);
        assert_eq!(info.pages, 59);
        assert_eq!(info.width, 612);
        assert_eq!(info.height, 792);
        assert_eq!(info.unit, PDFUnit::Px);
    }
}
