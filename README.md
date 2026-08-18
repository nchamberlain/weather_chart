# weatherChart
This project is designed to extract data from the city temperature database and produce the relavent charts. There are four existing types of charts:

 - Bar Charts
 - Line Charts
 - Temp-Date Charts
 - Temp-Range-Frequency Charts

## Bar Charts

Bar charts display the average temperature by period (week, 2-week, month) for exactly 1 year. Each entry in the chart is composed of two parts, the low temp average (darker colors in lower part of bar) and the high temp average (lighter colors). Each part (high temp or low temp) is color coded based on its temperature range (red for hottest and blue for coldest).

![barchart](imgs/barchart.jpg)

## Line Charts

Line charts display the average and median tempertures by period (week, 2-week, month) for exactly 1 year. Line charts show only 2 lines: red lines with bullets for that period's average high temperature and blue lines with bullets for that period's average low temperature. The high temp line and low temp lines are also annotated with the median high and median low tempertures as short bars.

![linechart](imgs/linechart.jpg)

## Top 10% Temperatures Charts

Each city has 4 Top 10% charts that cover the entire time period for which temperature data was captured. These charts show the Hottest and Coolest daytime temperatures and the Coldest and Warmest nighttime temperatures.

![temp-dateChart](imgs/tempdatechart.jpg)

## Weekly Average Temperature Range Frequency Charts

Each city has a daytime and a nighttime weekly average Temperature Range Frequency Chart. There are 9 temperture ranges: 100 and hotter, 90-99, 80-89, 70-79, 60-69, 50-59, 40-49, 32-39, and Freezing and below. Each temperature range is represented by a vertical box that shows the number of weeks spent in that temperature range for each year. In the example, Honolulu in 1966 had 4 weeks that averaged in the 70's, 43 weeks in the 80's, and 5 weeks in the 90's. The temperature ranges are color coded but ranges that had 0 weeks are white.

![temp-freqChart](imgs/tempfreqchart.png)


## Discussion of Bar and Line Charts

Right now the bar charts and line charts are too fine-grained to load onto the web and become way too cumbersome to try to make available on web site. Not to mention taking up way to much space.

For the bar charts and line charts, one chart is generated for weekly, fortnightly, and monthly averages for each year for each city. In other words:

`2 types x 3 charts x 100 years x 100 cities = 60,000 charts`

And they don't really tell you anything other than for one year the residents of one city recorded the temperatures that are averaged into the charts. Great for proof of work but we don't learn much from them. Way too many data points to be able to draw any conclusions about climate change.

## Discussion of Videos
To make the data in these charts more accessible, only videos will be presented. The individual charts will be combined into mp4 videos that show each individual chart as one frame. This will allow any user to view the foundation data as a single unit and watch the years, starting and stopping as desired. A fast and slow speed video will be provided for weekly, fortnightly, and monthly averages for bar charts and line charts for approximately 100 cities. In other words:

`2 speeds x 2 types x 3 charts x 100 cities = 1,200`

That's still a lot of files, but they'll only be 2% as many as the individual files. Probably will use some kind of choices list (fast or slow, bar or line, wk/fort/month) once you have selected the desired city.

## Discussion of Top 10% Charts

To gain greater insight into the issues, we need to see the complete span of years for which we have temperature data. This allows us to visually compare the earliest temperatures with the latest temperatures and detect any trends.

The Top 10% Charts select approximately 10% of the daily temperature records and plots them by date as individual dots on the chart. Areas with more dots with be more strongly colored. The hot and cold extremes for daytime temperatures are separately plotted, and the hot and cold extremes for nighttime temperatures are also separately plotted for a grand total of 4 charts.

It is important to note that these are individual daily records and NOT based on calculated averages. This helps to remove the potential for calculations to somehow skew the data presentation.

These charts have to be extremely large to accomodate the 100 - 150 years covered for each city. To spread the temperature dots as much as possible, these charts can be 10,000 pixels wide (which are hard to view). Even with the wide chart format, when the same temperatures happened in the same week or two, they would end up printing as the same dot. To prevent this misleading display, close date temperatures are accumulated and printed as proportionally larger dots.

In general, we see more of the top 10% hottest temperatures in the 21st century and the more of the top 10% coldest temperatures in the 19th and 20th centuries. We also see "jackpot" temperatures, where conditions are exactly right to produce extreme temperatures that are 4 - 8 degrees warmer or cooler in one or a couple of years. 

The Top 10% charts show part of why it is so hard to detect climate change looking at daily temperatures or average temperatures: Some of the hottest temperatures will appear in the 19th century and some of the coldest will show in the 21st. Our daily temperatures are determined by a big chunk of randomness and by the temperature "context" of the preceding few days, disguising the overall trends.

These charts also display "columns" of temperatures that cluster around certain years. For example, the Texas and Oklahoma charts show some of their hottest temperatures happened in the 1930s, when the dust bowl blew thousands of farmers off their lands. 

By selecting 10% of the hottest and coldest daytime temps and nighttime temps, we can say with some confidence that 80% of the daytime and night time temperatures will between them. For example, if 10% of the hottest daytime temps for LA are above 93 degrees and 10% of the coldest daytime temps are below 51 degrees, we can assume that 80% of the days will be between 51 and 93 degrees.

## Discussion of Temperature Range Frequency Charts 

These charts show the number of weeks in various temperature ranges by year for each city. Each year covered by the data has a stack of 9 white boxes representing the 9 temperature ranges tracked:
- 32 and below (freezing)
- 33 - 39
- 40 - 49
- 50 - 59
- 60 - 69
- 70 - 79
- 80 - 89
- 90 - 99
- 100 and above

Each temperature range has a color associated with it, ranging from blue to green to orange to red representing coldest to warmest. Each temperature range box is filled with its color in proportion to how many weeks the temperature was in that range: 0 weeks will be completely white; 26 weeks will be half filled; 52 weeks will be completely filled. 

If you add the numbers next to each range for a single year, the total should equal 52 for the 52 weeks of the year. Some data sets used have incomplete records for some years (for example, the daily temperature data for March 1983 may be missing), which will lead to the number of weeks to equal less than 52 weeks. In no case, however, should the total be greater than 52 because the calendar weekdays are ignored, making the first week of every year always the first thru the seventh, the second week the eighth thru the fourteenth, and so on. Week 52 will have 8 days except for leap years when it has 9 days.

## Future Plans

Also planned is a "this day compared to average, median, and min-max temperatures" feature. This will require moving the calculations onto the server to allow dynamic production of the charted output, but it would be fun to know how today's temperture compares to the average temps by decade and over all temps, as well as how today compares to the median temp for that day (equal number of days higher and lower) as well as the range of temps (absolute highest and lowest) for that day. 

Maybe not useful in the climate change narrative, but fun to know. 

Hmmm, this might need to be a separate app & website. Same data but separate websites. Sister sites.