# 菲律宾旅游地点预报覆盖

在“添加地点”里输入中文名、英文名或收录的别名即可搜索。清空输入框可浏览离线目录。每个结果显示中英文地名；选中后用该地点自己的经纬度查询 PAGASA PANaHON。离线可查地点，更新天气需要网络。

## 核验结果

2026 年 11 月 20 日逐坐标查询了以下 123 个新增地点。每个地点都取得了匹配的坐标回显、当前时段温度、24 个逐时预报和 5 个逐日预报。表中的温度是核验当时的样本。马尼拉、宿务、达沃及长滩岛仍保留原来的地点标识和坐标。

这些数据是地点附近的**大气预报**。风况和降水可以帮助安排行程；潜点的海流、水下能见度、浪高及船班通航情况需要另查当地海事通告。部分离岸点的官方接口没有返回市镇或省区。无法确定预警所属地区时，应用只匹配明确发布给全国的公告。[PAGASA 海上大风警报](https://www.pagasa.dost.gov.ph/marine/gale-warning)

正式包在运行 Android 17 的 Pixel 10a 上完成了 254 次实机搜索。对每个地点分别输入完整中文名和英文名，127 个地点都在结果列表中命中。另将这些地点的官方响应逐一送入应用的天气解析器，全部取得当前温度、24 小时和 5 日预报。实机选中图巴塔哈杰西比兹利礁和圭亚姆岛时，两个地点的首页均可刷新。

## 怎样选择地点

“民都洛岛”和“巴拉望岛”各有一个岛内参考坐标。查询海滩、镇区、泻湖或沉船潜点时，优先搜具体地点，尤其是大岛。地名相近的阿波岛（Apo Island）和阿波礁（Apo Reef）也有各自的结果与坐标。

预警使用 PAGASA 随该坐标返回的地名。长滩岛通常回显 Malay / Aklan，可匹配 Aklan 公告。部分独立市（例如碧瑶或公主港）可能只有市名或大区、没有省名；为避免无后缀城镇名与外省重名，此时会按确认的大区和全国标记筛选，不能保证显示只写裸市名或邻省名的公告。离岸点若连地区也没有，则只能匹配全国公告；该状态无法说明附近海域没有地方预警。

## 点位来源

下表逐项链接对应的地点资料。GeoNames 地名及坐标采用 CC BY 4.0，本项目整理了当地名称、中文别名和检索形式。另一些潜点和地标按各行所列的保护区、潜水机构、旅行测绘和观测资料定位。[GeoNames](https://www.geonames.org/about.html) · [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)

## 吕宋及周边、巴拉望和民都洛

| 搜索地点 | 坐标（纬度，经度） | PAGASA 回显地区 | 核验温度 |
| --- | --- | --- | --- |
| [爱妮岛镇 · El Nido](https://www.geonames.org/1713917/) | [11.18583, 119.39556](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.18583&lon=119.39556) | El Nido / Palawan / Mimaropa | 27 °C |
| [米尼洛岛 · Miniloc Island](https://www.geonames.org/1699567/) | [11.15056, 119.315](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.15056&lon=119.315) | El Nido / Palawan / Mimaropa | 29 °C |
| [卡德劳岛 · Cadlao Island](https://www.geonames.org/1721165/) | [11.21639, 119.35333](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.21639&lon=119.35333) | El Nido / Palawan / Mimaropa | 28 °C |
| [马廷洛克岛 · Matinloc Island](https://www.geonames.org/1700285/) | [11.17833, 119.28417](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.17833&lon=119.28417) | El Nido / Palawan / Mimaropa | 29 °C |
| [爱妮岛直升机岛 · Dilumacad Island](https://www.geonames.org/1714859/) | [11.20111, 119.33806](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.20111&lon=119.33806) | El Nido / Palawan / Mimaropa | 29 °C |
| [巴奎特湾 · Bacuit Bay](https://www.geonames.org/1729472/) | [11.07889, 119.38806](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.07889&lon=119.38806) | El Nido / Palawan / Mimaropa | 27 °C |
| [科隆镇 · Coron Town](https://www.geonames.org/1716834/) | [11.9986, 120.2043](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.9986&lon=120.2043) | Coron / Palawan / Mimaropa | 28 °C |
| [科隆岛 · Coron Island](https://www.geonames.org/1716820/) | [11.9286, 120.2412](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.9286&lon=120.2412) | Coron / Palawan / Mimaropa | 29 °C |
| [布桑加岛 · Busuanga Island](https://www.geonames.org/1722262/) | [12.1033, 120.0837](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.1033&lon=120.0837) | Coron / Palawan / Mimaropa | 26 °C |
| [库利昂岛 · Culion Island](https://www.geonames.org/1716527/) | [11.8352, 119.9933](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.8352&lon=119.9933) | Culion / Palawan / Mimaropa | 28 °C |
| [卡劳伊特岛 · Calauit Island](https://www.geonames.org/1720459/) | [12.30139, 119.89861](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.30139&lon=119.89861) | Busuanga / Palawan / Mimaropa | 28 °C |
| [马尔卡普亚岛 · Malcapuya Island](https://www.geonames.org/1702878/) | [11.79, 120.115](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.79&lon=120.115) | Culion / Palawan / Mimaropa | 29 °C |
| [桑加特岛 · Sangat Island](https://www.geonames.org/1683211/) | [11.9766, 120.0719](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.9766&lon=120.0719) | Coron / Palawan / Mimaropa | 28 °C |
| [吕松岛（科隆湾） · Lusong Island](https://www.geonames.org/1704926/) | [11.9785, 120.0204](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.9785&lon=120.0204) | Coron / Palawan / Mimaropa | 28 °C |
| [帕马利肯岛 · Pamalican Island (Cuyo)](https://www.geonames.org/1695935/) | [11.3557, 120.7291](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.3557&lon=120.7291) | Cuyo / Palawan / Mimaropa | 29 °C |
| [利纳帕坎岛 · Linapacan Island](https://www.geonames.org/1706241/) | [11.4441, 119.8291](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.4441&lon=119.8291) | Linapacan / Palawan / Mimaropa | 29 °C |
| [巴拉望圣维森特 · San Vicente, Palawan](https://www.geonames.org/1687474/) | [10.53033, 119.25423](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.53033&lon=119.25423) | Roxas / Palawan / Mimaropa | 27 °C |
| [公主港 · Puerto Princesa](https://www.geonames.org/1692685/) | [9.73917, 118.73528](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.73917&lon=118.73528) | Puerto Princesa / Mimaropa | 27 °C |
| [本田湾 · Honda Bay](https://www.geonames.org/1711539/) | [9.90056, 118.78278](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.90056&lon=118.78278) | Puerto Princesa / Mimaropa | 28 °C |
| [巴拉望沙邦海滩 · Sabang Beach, Palawan](https://www.geonames.org/7385149/) | [10.19672, 118.8934](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.19672&lon=118.8934) | Puerto Princesa / Mimaropa | 27 °C |
| [阿波礁（民都洛） · Apo Reef](https://www.geonames.org/1730318/) | [12.6758, 120.475](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.6758&lon=120.475) | Sablayan / Occidental Mindoro / Mimaropa | 29 °C |
| [阿波礁阿波岛（民都洛） · Apo Island, Apo Reef](https://www.geonames.org/1730328/) | [12.6601, 120.4144](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.6601&lon=120.4144) | Sablayan / Occidental Mindoro / Mimaropa | 30 °C |
| [加莱拉港 · Puerto Galera](https://www.geonames.org/1692688/) | [13.50223, 120.95166](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.50223&lon=120.95166) | Puerto Galera / Oriental Mindoro / Mimaropa | 27 °C |
| [加莱拉港沙邦 · Sabang, Puerto Galera](https://www.geonames.org/1691358/) | [13.5212, 120.9819](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.5212&lon=120.9819) | Puerto Galera / Oriental Mindoro / Mimaropa | 27 °C |
| [民都洛北潘丹岛 · North Pandan Island](https://www.geonames.org/1697536/) | [12.85791, 120.74979](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.85791&lon=120.74979) | Sablayan / Occidental Mindoro / Mimaropa | 28 °C |
| [阿尼洛（八打雁） · Anilao, Batangas](https://www.geonames.org/1730661/) | [13.76036, 120.92927](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.76036&lon=120.92927) | Mabini / Batangas / Calabarzon | 25 °C |
| [佛得岛（八打雁） · Verde Island](https://www.geonames.org/1680054/) | [13.5511, 121.0765](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.5511&lon=121.0765) | Batangas / Calabarzon | 28 °C |
| [八打雁草帽岛 · Sombrero Island, Batangas](https://www.geonames.org/1685821/) | [13.69832, 120.82918](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.69832&lon=120.82918) | Tingloy / Batangas / Calabarzon | 27 °C |
| [八打雁市 · Batangas City](https://www.geonames.org/1726280/) | [13.7567, 121.0584](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.7567&lon=121.0584) | Batangas / Calabarzon | 24 °C |
| [大雅台 · Tagaytay](https://www.geonames.org/1684553/) | [14.09532, 120.93355](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=14.09532&lon=120.93355) | Tagaytay / Cavite / Calabarzon | 23 °C |
| [苏比克湾 · Subic Bay](https://www.geonames.org/1685574/) | [14.78997, 120.23254](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=14.78997&lon=120.23254) | Morong / Bataan / Central Luzon | 25 °C |
| [百岛群岛 · Hundred Islands (Alaminos)](https://www.geonames.org/1711466/) | [16.21163, 120.04537](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=16.21163&lon=120.04537) | Alaminos / Pangasinan / Ilocos Region | 28 °C |
| [皮纳图博火山 · Mount Pinatubo](https://www.geonames.org/1693517/) | [15.15892, 120.37161](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=15.15892&lon=120.37161) | Zambales / Central Luzon | 22 °C |
| [巴莱尔 · Baler](https://www.geonames.org/1728264/) | [15.7589, 121.5607](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=15.7589&lon=121.5607) | Baler / Aurora / Central Luzon | 25 °C |
| [拉乌尼翁圣胡安 · San Juan, La Union](https://www.geonames.org/1689249/) | [16.66861, 120.34139](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=16.66861&lon=120.34139) | San Juan / La Union / Ilocos Region | 27 °C |
| [碧瑶 · Baguio](https://www.geonames.org/1728930/) | [16.41639, 120.59306](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=16.41639&lon=120.59306) | Baguio / Cordillera Administrative Region | 20 °C |
| [萨加达 · Sagada](https://www.geonames.org/1691184/) | [17.084, 120.8996](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=17.084&lon=120.8996) | Sagada / Mountain Province / Cordillera Administrative Region | 18 °C |
| [巴纳韦 · Banaue](https://www.geonames.org/1727529/) | [16.91356, 121.06184](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=16.91356&lon=121.06184) | Banaue / Ifugao / Cordillera Administrative Region | 17 °C |
| [伊富高巴塔德村 · Batad, Ifugao](https://www.geonames.org/1726331/) | [16.93306, 121.13644](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=16.93306&lon=121.13644) | Banaue / Ifugao / Cordillera Administrative Region | 18 °C |
| [维甘 · Vigan](https://www.geonames.org/1679980/) | [17.57472, 120.38694](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=17.57472&lon=120.38694) | Vigan / Ilocos Sur / Ilocos Region | 26 °C |
| [拉瓦格 · Laoag](https://www.geonames.org/1707404/) | [18.1978, 120.5957](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=18.1978&lon=120.5957) | Laoag / Ilocos Norte / Ilocos Region | 26 °C |
| [帕古普德 · Pagudpud](https://www.geonames.org/1696482/) | [18.5616, 120.7868](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=18.5616&lon=120.7868) | Pagudpud / Ilocos Norte / Ilocos Region | 24 °C |
| [帕劳伊岛 · Palaui Island](https://www.geonames.org/1696184/) | [18.54945, 122.13696](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=18.54945&lon=122.13696) | Santa Ana / Cagayan / Cagayan Valley | 25 °C |
| [巴丹群岛·巴丹岛 · Batan Island (Batanes)](https://www.geonames.org/1726264/) | [20.42135, 121.96047](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=20.42135&lon=121.96047) | Mahatao / Batanes / Cagayan Valley | 24 °C |
| [巴丹群岛·巴斯科 · Basco, Batanes](https://www.geonames.org/1726449/) | [20.44865, 121.97017](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=20.44865&lon=121.97017) | Basco / Batanes / Cagayan Valley | 24 °C |
| [沙坦岛 · Sabtang Island](https://www.geonames.org/1691257/) | [20.30982, 121.86592](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=20.30982&lon=121.86592) | Sabtang / Batanes / Cagayan Valley | 24 °C |
| [伊特巴亚特岛 · Itbayat Island](https://www.geonames.org/1710455/) | [20.76253, 121.83769](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=20.76253&lon=121.83769) | Itbayat / Batanes / Cagayan Valley | 24 °C |
| [马荣火山 · Mayon Volcano](https://www.geonames.org/1699962/) | [13.25474, 123.68506](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.25474&lon=123.68506) | Malilipot / Albay / Bicol Region | 23 °C |
| [董索尔 · Donsol](https://www.geonames.org/1714410/) | [12.9055, 123.5949](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.9055&lon=123.5949) | Donsol / Sorsogon / Bicol Region | 26 °C |
| [蒂考岛 · Ticao Island](https://www.geonames.org/1682362/) | [12.5291, 123.6981](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.5291&lon=123.6981) | Masbate / Bicol Region | 26 °C |
| [卡拉莫安镇 · Caramoan Town](https://www.geonames.org/1718508/) | [13.7707, 123.8631](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.7707&lon=123.8631) | Caramoan / Camarines Sur / Bicol Region | 26 °C |
| [卡拉瓜斯群岛·蒂纳加岛 · Tinaga Island, Calaguas](https://www.geonames.org/1681998/) | [14.4651, 122.9343](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=14.4651&lon=122.9343) | Vinzons / Camarines Norte / Bicol Region | 28 °C |
| [图巴塔哈群礁 · Tubbataha Reefs](https://www.geonames.org/11524888/) | [8.85, 119.93333](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=8.85&lon=119.93333) | Palawan / Mimaropa | 29 °C |
| [图巴塔哈杰西比兹利礁 · Jessie Beazley Reef](https://img1.wsimg.com/blobby/go/a6e8679e-19dc-4a2b-8c4f-4ec1c667e7e3/downloads/TRNP%20ERM%20Report%202023-compressed.pdf#page=169) | [9.04388, 119.81595](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.04388&lon=119.81595) | 未提供 | 29 °C |
| [公主港地下河国家公园 · Puerto-Princesa Subterranean River National Park](https://whc.unesco.org/en/list/652/maps/) | [10.16667, 118.91667](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.16667&lon=118.91667) | Puerto Princesa / Mimaropa | 26 °C |
| [爱妮岛大泻湖 · Big Lagoon, El Nido](https://www.belka.travel/en/places/big-lagoon-el-nido/) | [11.15423, 119.32093](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.15423&lon=119.32093) | El Nido / Palawan / Mimaropa | 29 °C |
| [爱妮岛小泻湖 · Small Lagoon, El Nido](https://www.belka.travel/en/places/small-lagoon-el-nido/) | [11.15636, 119.32139](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.15636&lon=119.32139) | El Nido / Palawan / Mimaropa | 29 °C |
| [科隆凯央根湖 · Kayangan Lake](https://www.wikidata.org/wiki/Q125206346) | [11.95726, 120.22623](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.95726&lon=120.22623) | Coron / Palawan / Mimaropa | 28 °C |
| [科隆梭鱼湖 · Barracuda Lake](https://freediving-coron.com/attractions/barracuda-lake) | [11.9565, 120.2169](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.9565&lon=120.2169) | Coron / Palawan / Mimaropa | 28 °C |
| [科隆双子泻湖 · Twin Lagoon, Coron](https://freediving-coron.com/attractions/twin-lagoon) | [11.9479, 120.2108](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.9479&lon=120.2108) | Coron / Palawan / Mimaropa | 29 °C |
| [秋津洲号沉船 · Akitsushima Wreck](https://www.padi.com/dive-site/philippines/akitsushima/) | [11.98684, 119.97346](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.98684&lon=119.97346) | Busuanga / Palawan / Mimaropa | 28 °C |
| [奥林匹亚丸沉船 · Olympia Maru Wreck](https://www.padi.com/dive-site/philippines/olympia-maru/) | [11.97073, 120.06125](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.97073&lon=120.06125) | Coron / Palawan / Mimaropa | 28 °C |
| [伊良湖号沉船 · Irako Wreck](https://www.padi.com/dive-site/philippines/irako-2/) | [11.96821, 120.04027](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.96821&lon=120.04027) | Coron / Palawan / Mimaropa | 28 °C |
| [阿尼洛大教堂潜点 · Cathedral Rock, Anilao](https://thescubadirectory.com/divesiteprofile/Cathedral-Rock-Anilao-Philippines-Dive-Site) | [13.72429, 120.87598](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.72429&lon=120.87598) | Mabini / Batangas / Calabarzon | 26 °C |
| [加莱拉港基利马峭壁 · Kilima Wall](https://www.padi.com/dive-site/philippines/kilima-wall/) | [13.51834, 120.99170](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.51834&lon=120.99170) | Puerto Galera / Oriental Mindoro / Mimaropa | 27 °C |
| [加莱拉港鲨鱼洞 · Shark Cave, Puerto Galera](https://www.padi.com/dive-site/philippines/sharks-cave-2/) | [13.52275, 120.99280](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.52275&lon=120.99280) | Puerto Galera / Oriental Mindoro / Mimaropa | 27 °C |
| [爱妮岛坦巴拉南潜点 · Tambalanang Dive Site](https://www.padi.com/dive-site/philippines/tambalanang/) | [11.21755, 119.32594](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.21755&lon=119.32594) | El Nido / Palawan / Mimaropa | 29 °C |
| [纳克潘海滩 · Nacpan Beach](https://www.belka.travel/en/places/nacpan-beach/) | [11.32119, 119.42776](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.32119&lon=119.42776) | El Nido / Palawan / Mimaropa | 28 °C |
| [巴顿港 · Port Barton](https://steller.co/d/port-barton-4AV7JZhV2ekuYv2xWrGvLsj/) | [10.41139, 119.17777](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.41139&lon=119.17777) | Roxas / Palawan / Mimaropa | 26 °C |
| [巴拉望岛 · Palawan Island](https://www.geonames.org/1696180/) | [9.53153, 118.46149](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.53153&lon=118.46149) | Palawan / Mimaropa | 23 °C |
| [民都洛岛 · Mindoro Island](https://www.geonames.org/1699590/) | [12.9304, 121.0947](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.9304&lon=121.0947) | Sablayan / Occidental Mindoro / Mimaropa | 19 °C |
| [塔尔湖 · Taal Lake](https://www.geonames.org/1685141/) | [13.9824, 121.017](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=13.9824&lon=121.017) | San Nicolas / Batangas / Calabarzon | 25 °C |
| [普洛格山 · Mount Pulag](https://www.geonames.org/1692545/) | [16.59778, 120.89914](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=16.59778&lon=120.89914) | Ifugao / Cordillera Administrative Region | 15 °C |
| [图巴塔哈阿莫斯岩 · Amos Rock (Tubbataha)](https://medtrix.fr/wp-content/uploads/2025/04/GOMBESSA_TUBBATAHA_230225_V7s.pdf#page=11) | [8.845417, 119.8974](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=8.845417&lon=119.8974) | Palawan / Mimaropa | 29 °C |
| [图巴塔哈鲨鱼机场 · Shark Airport (Tubbataha)](https://medtrix.fr/wp-content/uploads/2025/04/GOMBESSA_TUBBATAHA_230225_V7s.pdf#page=11) | [8.92527, 120.00795](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=8.92527&lon=120.00795) | Palawan / Mimaropa | 29 °C |
| [图巴塔哈马来亚沉船 · Malayan Wreck (Tubbataha)](https://medtrix.fr/wp-content/uploads/2025/04/GOMBESSA_TUBBATAHA_230225_V7s.pdf#page=11) | [8.8827, 119.8888](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=8.8827&lon=119.8888) | Palawan / Mimaropa | 29 °C |

## 维萨亚斯和棉兰老

| 搜索地点 | 坐标（纬度，经度） | PAGASA 回显地区 | 核验温度 |
| --- | --- | --- | --- |
| [薄荷岛 · Bohol Island](https://www.geonames.org/1724399/) | [9.83333, 124.16667](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.83333&lon=124.16667) | Bohol / Central Visayas | 25 °C |
| [邦劳岛 · Panglao Island](https://www.geonames.org/1695260/) | [9.59639, 123.80861](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.59639&lon=123.80861) | Dauis / Bohol / Central Visayas | 27 °C |
| [阿罗娜海滩 · Alona Beach](https://www.geonames.org/11054339/) | [9.54797, 123.77156](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.54797&lon=123.77156) | Panglao / Bohol / Central Visayas | 28 °C |
| [巴里卡萨岛 · Balicasag Island](https://www.geonames.org/1728180/) | [9.51696, 123.68436](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.51696&lon=123.68436) | Panglao / Bohol / Central Visayas | 29 °C |
| [帕米拉坎岛 · Pamilacan Island](https://www.geonames.org/1695879/) | [9.49265, 123.92366](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.49265&lon=123.92366) | Bohol / Central Visayas | 29 °C |
| [卡比劳岛 · Cabilao Island](https://www.geonames.org/1721604/) | [9.88056, 123.77028](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.88056&lon=123.77028) | Bohol / Central Visayas | 28 °C |
| [墨宝 · Moalboal](https://www.geonames.org/1699454/) | [9.94333, 123.39917](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.94333&lon=123.39917) | Moalboal / Cebu / Central Visayas | 25 °C |
| [马拉帕斯卡岛 · Malapascua Island](https://www.geonames.org/1703054/) | [11.33689, 124.11571](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.33689&lon=124.11571) | Cebu / Central Visayas | 28 °C |
| [加托岛（宿务） · Gato Island (Cebu)](https://www.geonames.org/1713120/) | [11.4468, 124.02058](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.4468&lon=124.02058) | Cebu / Central Visayas | 28 °C |
| [阿波岛（东内格罗斯） · Apo Island (Negros Oriental)](https://www.geonames.org/1730331/) | [9.0794, 123.271](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.0794&lon=123.271) | Dauin / Negros Oriental / Negros Island Region | 29 °C |
| [道因 · Dauin](https://www.geonames.org/1715369/) | [9.1914, 123.2654](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.1914&lon=123.2654) | Dauin / Negros Oriental / Negros Island Region | 27 °C |
| [杜马盖地 · Dumaguete](https://www.geonames.org/1714201/) | [9.30646, 123.30769](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.30646&lon=123.30769) | Dumaguete / Negros Oriental / Negros Island Region | 27 °C |
| [锡基霍尔岛 · Siquijor Island](https://www.geonames.org/1686064/) | [9.1975, 123.59639](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.1975&lon=123.59639) | Larena / Siquijor / Negros Island Region | 27 °C |
| [锡亚高岛 · Siargao Island](https://www.geonames.org/1686882/) | [9.87083, 126.05111](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.87083&lon=126.05111) | Pilar / Surigao del Norte / Caraga | 26 °C |
| [卢纳将军镇（锡亚高） · General Luna (Siargao)](https://www.geonames.org/1713046/) | [9.78375, 126.15657](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.78375&lon=126.15657) | General Luna / Surigao del Norte / Caraga | 28 °C |
| [布卡斯格兰德岛 · Bucas Grande Island](https://www.geonames.org/1723575/) | [9.67361, 125.94861](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.67361&lon=125.94861) | Socorro / Surigao del Norte / Caraga | 28 °C |
| [甘米银岛 · Camiguin Island](https://www.geonames.org/1719692/) | [9.17454, 124.71856](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.17454&lon=124.71856) | Mambajao / Camiguin / Northern Mindanao | 26 °C |
| [曼提格岛 · Mantigue Island](https://www.geonames.org/1703766/) | [9.17639, 124.82556](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.17639&lon=124.82556) | Mahinog / Camiguin / Northern Mindanao | 27 °C |
| [白岛（甘米银） · White Island (Camiguin)](https://www.geonames.org/1699899/) | [9.25855, 124.65501](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.25855&lon=124.65501) | Mambajao / Camiguin / Northern Mindanao | 27 °C |
| [萨马尔岛（达沃湾） · Samal Island (Davao Gulf)](https://www.geonames.org/1690655/) | [7.04778, 125.74417](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=7.04778&lon=125.74417) | Samal / Davao del Norte / Davao Region | 26 °C |
| [塔利库德岛 · Talikud Island](https://www.geonames.org/1683942/) | [6.93218, 125.69872](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=6.93218&lon=125.69872) | Samal / Davao del Norte / Davao Region | 26 °C |
| [卡朗加曼岛 · Kalanggaman Island](https://www.geonames.org/1720621/) | [11.11465, 124.25117](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.11465&lon=124.25117) | Palompon / Leyte / Eastern Visayas | 28 °C |
| [利马萨瓦岛 · Limasawa Island](https://www.geonames.org/1706367/) | [9.931, 125.0744](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.931&lon=125.0744) | Limasawa / Southern Leyte / Eastern Visayas | 29 °C |
| [莱特岛 · Leyte Island](https://www.geonames.org/1706802/) | [10.85957, 124.87607](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.85957&lon=124.87607) | La Paz / Leyte / Eastern Visayas | 23 °C |
| [麦克坦岛 · Mactan Island](https://www.geonames.org/1704245/) | [10.29194, 123.96806](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.29194&lon=123.96806) | Lapu-Lapu / Central Visayas | 27 °C |
| [班塔延岛 · Bantayan Island](https://www.geonames.org/1727033/) | [11.2054, 123.7398](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.2054&lon=123.7398) | Bantayan / Cebu / Central Visayas | 28 °C |
| [苏米龙岛 · Sumilon Island](https://www.geonames.org/1685299/) | [9.4325, 123.3893](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.4325&lon=123.3893) | Oslob / Cebu / Central Visayas | 28 °C |
| [奥斯洛布 · Oslob](https://www.geonames.org/1696989/) | [9.5211, 123.4315](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.5211&lon=123.4315) | Oslob / Cebu / Central Visayas | 27 °C |
| [奥兰戈岛 · Olango Island](https://www.geonames.org/1697232/) | [10.26611, 124.05694](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.26611&lon=124.05694) | Lapu-Lapu / Central Visayas | 28 °C |
| [吉马拉斯岛 · Guimaras Island](https://www.geonames.org/1712493/) | [10.57966, 122.5941](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.57966&lon=122.5941) | Jordan / Guimaras / Western Visayas | 25 °C |
| [丹胡甘岛 · Danjugan Island](https://www.geonames.org/1715682/) | [9.8729, 122.3789](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.8729&lon=122.3789) | Cauayan / Negros Occidental / Negros Island Region | 28 °C |
| [比里岛 · Biri Island](https://www.geonames.org/1724798/) | [12.66806, 124.37694](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=12.66806&lon=124.37694) | Biri / Northern Samar / Eastern Visayas | 27 °C |
| [塞布湖 · Lake Sebu](https://www.geonames.org/1687028/) | [6.23116, 124.70697](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=6.23116&lon=124.70697) | Lake Sebu / South Cotabato / Soccsksargen | 21 °C |
| [佩斯卡多岛潜点 · Pescador Island Dive Site](https://www.padi.com/dive-site/philippines/pescador-island/) | [9.92321, 123.34385](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.92321&lon=123.34385) | Moalboal / Cebu / Central Visayas | 27 °C |
| [莫纳德浅滩 · Monad Shoal](https://www.padi.com/dive-site/philippines/monad-shoal-thresher-shark-divers-mooring-line/) | [11.30754, 124.18757](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.30754&lon=124.18757) | Cebu / Central Visayas | 28 °C |
| [基穆德浅滩 · Kimud Shoal](https://www.padi.com/dive-site/philippines/kimud-shoal/) | [11.15464, 124.28879](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.15464&lon=124.28879) | Villaba / Leyte / Eastern Visayas | 27 °C |
| [巴里卡萨黑森林潜点 · Black Forest (Balicasag)](https://www.boholfundivers.com/black-forest) | [9.51483, 123.68893](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.51483&lon=123.68893) | Panglao / Bohol / Central Visayas | 29 °C |
| [图博德海洋保护区潜点 · Tubod Marine Sanctuary (Siquijor)](https://www.padi.com/dive-site/philippines/tubod/) | [9.14430, 123.50718](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.14430&lon=123.50718) | San Juan / Siquijor / Negros Island Region | 28 °C |
| [纳潘陶海洋保护区 · Napantao Marine Sanctuary](https://coralnet.ucsd.edu/source/1768/) | [10.06484, 125.13450](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=10.06484&lon=125.13450) | Southern Leyte / Eastern Visayas | 28 °C |
| [圭亚姆岛 · Guyam Island](https://www.s1expeditions.com/2015/03/185-island-hopping-siargao.html) | [9.76524, 126.16759](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.76524&lon=126.16759) | General Luna / Surigao del Norte / Caraga | 28 °C |
| [达库岛 · Daku Island](https://www.s1expeditions.com/2015/03/185-island-hopping-siargao.html) | [9.74291, 126.15505](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.74291&lon=126.15505) | General Luna / Surigao del Norte / Caraga | 28 °C |
| [裸岛（锡亚高） · Naked Island (Siargao)](https://www.s1expeditions.com/2015/03/185-island-hopping-siargao.html) | [9.73486, 126.12594](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=9.73486&lon=126.12594) | Dapa / Surigao del Norte / Caraga | 28 °C |
| [白沙滩（长滩岛） · White Beach (Boracay)](https://www.lakadpilipinas.com/2016/10/boracay-travel-guide-itinerary-budget.html) | [11.96555, 121.92126](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.96555&lon=121.92126) | Malay / Aklan / Western Visayas | 26 °C |
| [普卡贝壳海滩（长滩岛） · Puka Shell Beach (Boracay)](https://www.lakadpilipinas.com/2016/10/boracay-travel-guide-itinerary-budget.html) | [11.99661, 121.91525](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.99661&lon=121.91525) | Malay / Aklan / Western Visayas | 27 °C |
| [丁威德海滩（长滩岛） · Diniwid Beach (Boracay)](https://www.lakadpilipinas.com/2016/10/boracay-travel-guide-itinerary-budget.html) | [11.97606, 121.91179](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.97606&lon=121.91179) | Malay / Aklan / Western Visayas | 27 °C |
| [布拉波海滩（长滩岛） · Bulabog Beach (Boracay)](https://www.lakadpilipinas.com/2016/10/boracay-travel-guide-itinerary-budget.html) | [11.96563, 121.92889](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.96563&lon=121.92889) | Malay / Aklan / Western Visayas | 26 °C |
| [鳄鱼岛潜点（长滩岛） · Crocodile Island Dive Site (Boracay)](https://www.padi.com/dive-site/philippines/crocodile-island/) | [11.94941, 121.94907](https://api.mobile.panahon.gov.ph/api/v1/nowcast/dashboard?lat=11.94941&lon=121.94907) | Malay / Aklan / Western Visayas | 26 °C |
