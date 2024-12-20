# 中华人民共和国行政区划代码
# https://www.mca.gov.cn/n156/n186/index.html

from pathlib import Path
import re
import time
import json

import requests
from bs4 import BeautifulSoup

URL_INDEX = {
    2023: "https://www.mca.gov.cn/mzsj/xzqh/2023/202301xzqh.html",
    2022: "https://www.mca.gov.cn/mzsj/xzqh/2022/202201xzqh.html",
    2021: "https://www.mca.gov.cn/mzsj/xzqh/2021/20211201.html",
    2020: "https://www.mca.gov.cn/mzsj/xzqh/2020/20201201.html",
    2019: "https://www.mca.gov.cn/mzsj/xzqh/1980/2019/202002281436.html",
    2018: "https://www.mca.gov.cn/mzsj/xzqh/1980/201903/201903011447.html",
    2017: "https://www.mca.gov.cn/mzsj/xzqh/1980/201803/201803131454.html",
    2016: "https://www.mca.gov.cn/mzsj/xzqh/1980/201705/201705311652.html",
    2015: "https://www.mca.gov.cn/mzsj/tjbz/a/2015/201706011127.html",
    2014: "https://www.mca.gov.cn/images2/cws/201502/20150225163817214.html",
    2013: "https://www.mca.gov.cn/images2/cws/201404/20140404125552372.htm",
    2012: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201707271556.html",
    2011: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201707271552.html",
    2010: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220946.html",
    2009: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220943.html",
    2008: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220941.html",
    2007: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220939.html",
    2006: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220936.html",
    2005: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220935.html",
    2004: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220930.html",
    2003: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220928.html",
    2002: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220927.html",
    2001: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220925.html",
    2000: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220923.html",
    1999: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220921.html",
    1998: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220918.html",
    1997: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220916.html",
    1996: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220914.html",
    1995: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220913.html",
    1994: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220911.html",
    1993: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708041023.html",
    1992: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220910.html",
    1991: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708041020.html",
    1990: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708041018.html",
    1989: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708041017.html",
    1988: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220903.html",
    1987: "https://www.mca.gov.cn/mzsj/xzqh/1980/1980/201911180950.html",
    1986: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220859.html",
    1985: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220858.html",
    1984: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708220856.html",
    1983: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708160821.html",
    1982: "https://www.mca.gov.cn/mzsj/xzqh/1980/1980/201911180942.html",
    1981: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708041004.html",
    1980: "https://www.mca.gov.cn/mzsj/tjbz/a/201713/201708040959.html",
}

URL_CACHE_DIR = Path("data/url_cache")
URL_CACHE_DIR.mkdir(exist_ok=True, parents=True)


def get_text(name: str, url: str) -> str:
    cache_path = URL_CACHE_DIR / f"{name}.bin"

    if not cache_path.exists():
        time.sleep(1)
        print(f"Downloading {name}: {url}")
        response = requests.get(url)
        response.raise_for_status()
        cache_path.write_bytes(response.content)

    return cache_path.read_bytes().decode(errors="ignore")


def main():
    dataset = {}
    for year, url in URL_INDEX.items():
        print(f"Processing {year}")
        dataset[year] = {}

        html = get_text(str(year), url)
        soup = BeautifulSoup(html, "lxml")
        tr_list = soup.find_all("tr")
        tr_list = [[td for td in tr.find_all("td") if td.text != ""] for tr in tr_list]
        tr_list = [tr for tr in tr_list if len(tr) > 0]
        for tr in tr_list:
            if re.match(r"\d{6}", tr[0].text) is None:
                continue
            code = tr[0].text.strip()
            name = tr[1].text.strip()
            dataset[year][code] = name

    with open("src/region.json", "w") as f:
        json.dump(dataset, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
