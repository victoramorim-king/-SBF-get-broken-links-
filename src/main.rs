use mysql::*;
use mysql::prelude::*;
use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
struct SiteUrl {
    client: Option<String>,
    site_url: Option<String>
}

fn get_anchor_href_value(attribute: &str) -> &str  {
    let padrao = r#"https?://\S+"#;
        let regex = Regex::new(padrao).unwrap();
        let href_value = regex.captures(attribute).unwrap().get(0).unwrap();
        return href_value.as_str();

}

fn validate_link_regex(link: &str) -> bool {
    let regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
    let is_valid = regex.is_match(link);
    return is_valid;
}

fn is_broken_link(link: &str) -> bool{
    if reqwest::blocking::get(format!("{}", &link)).unwrap().status() == 404 {
        return true
    }
    return false; 
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mysql_url: &str = include!("../resources/mysql-access.txt"); // "mysql://root:password@localhost:3306/DatabaseName"
    let pool = Pool::new(mysql_url)?;
    let mut conn = pool.get_conn()?;

    // // conn.query_drop(
    // //     r"CREATE TABLE seo_sites_urls (
    // //         id int key NOT NULL AUTO_INCREMENT,
    // //         client text,
    // //         site_url text
    // //     )")?;


    // // let seo_sites_urls = vec![
    // //     SiteUrl { client: Some("Jessica Lestayo".into()), site_url: Some("https://jessicaleistayo.com/".into()) },
    // // ];

    // // conn.exec_batch(
    // //     r"INSERT INTO seo_sites_urls (client, site_url)
    // //       VALUES (:client, :site_url)",
    // //     seo_sites_urls.iter().map(|p| params! {
    // //         "client" => &p.client,
    // //         "site_url" => &p.site_url,
    // //     })
    // // )?;

    let selected_sites_url = conn
        .query_map(
            "SELECT client, site_url from seo_sites_urls",
            |(client, site_url)| {
                SiteUrl {client, site_url }
            }
        )?;


    for site in selected_sites_url {
        let sitemaps_list_xml = reqwest::blocking::get(
            format!("{}sitemap_index.xml", site.site_url.unwrap()),
        ).unwrap().text().unwrap();

        let sitemap_list_xml_tree = roxmltree::Document::parse(sitemaps_list_xml.as_str()).unwrap();

        for sitemap_list_node in sitemap_list_xml_tree.descendants() {
            if sitemap_list_node.is_element() && sitemap_list_node.tag_name().name() == "loc"  {

                let sitemap = reqwest::blocking::get(
                    format!("{}", sitemap_list_node.text().unwrap()),
                ).unwrap().text().unwrap();

                let sitemap_xml_tree = roxmltree::Document::parse(sitemap.as_str()).unwrap();

                for sitemap_node in sitemap_xml_tree.descendants() {

                    if sitemap_node.is_element() && sitemap_node.tag_name().name() == "loc"  {

                        let  webpage_link = sitemap_node.text().unwrap();

                        if !webpage_link.contains("?") && !webpage_link.contains("wp-content") {
                            // println!("{:?}", node.text().unwrap());
                            //
                            let webpage_body = reqwest::blocking::get(
                                format!("{}", webpage_link),
                            ).unwrap().text().unwrap();

                            let webpage_body_tree = scraper::Html::parse_document(&webpage_body);

                            let anchor_tag = scraper::Selector::parse("a").unwrap();

                            let all_anchor_tags = webpage_body_tree.select(&anchor_tag).map(|x| x.value());

                            for anchor_tag in all_anchor_tags { 
                                    if format!("{:?}", anchor_tag).contains("href=") {
                                        let anchor_tag_attributes: Vec<String> = anchor_tag.attrs.values()
                                            .map(|v| v.to_string())
                                            .collect();

                                        for attribute in anchor_tag_attributes {
                                            if validate_link_regex(&attribute){
                                                let anchor_href_value = get_anchor_href_value(&attribute);

                                                if is_broken_link(anchor_href_value) {
                                                    print!(
                                                        "Broken link: {:?} | in page: {:?}\n",
                                                        anchor_href_value,
                                                        webpage_link

                                                    )
                                                }
                                            }
                                        }

                                    }
                                };
                        }
                    }
                }
            }
        }    
    }

    Ok(())
}

