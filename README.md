# weatherChart
This project is designed to extract data from the city temperature database and produce the relavent charts. There are three existing types of charts:

 - Bar Charts
 - Line Charts
 - Temp-Date Charts

![barchart](ref_stuff/barchart.jpg)

![linechart](ref_stuff/linechart.jpg)

![temp-dateChart](ref_stuff/tempdatechart.jpg)

More analytical charts will be added soon.

Right now the bar charts and line charts are too fine-grained to load onto the web and become way too cumbersome to try to make available on web site. Not to mention taking up way to much space.

For the bar charts and line charts, one chart is generated for weekly, fortnightly, and monthly averages for each year for each city. In other words:

`2 types x 3 charts x 100 years x 100 cities = 60,000 charts`

And they don't really tell you anything other than for one year the residents of one city recorded the temperatures that are averaged into the charts. Great for proof of work but we don't learn much from them. Way too many data points to be able to draw any conclusions about climate change.

To make the data in these charts more accessible, only videos will be presented. The individual charts will be combined into mp4 videos that show each individual chart as one frame. This will allow any user to view the foundation data as a single unit and watch the years, starting and stopping as desired. A fast and slow speed video will be provided for weekly, fortnightly, and monthly averages for bar charts and line charts for approximately 100 cities. In other words:

`2 speeds x 2 types x 3 charts x 100 cities = 1,200`

That's still a lot of files, but they'll only be 2% of the individual files. Probably will use some kind of choices list (fast or slow, bar or line, wk/fort/month) once you have selected the desired city.

The emphasis is moving towards more analytical charts because it's way too easy to get lost in the data and not learn anything from the data. 

The last chart type, the temp-date chart, selects approximately 10% of the daily temperature records and plots them by date. The hot and cold extremes for daytime temperatures are separately plotted, and the hot and cold extremes for nighttime temperatures are also separately plotted for a grand total of 4 charts.

It is important to note that these are individual daily records and NOT based on calculated averages. This helps to remove the potential for calculations to somehow skew the data presentation.

These charts are monsters to accomodate the approximately 100 years covered for each city. To keep the individual temperature data points from stacking on top of each other, these charts are 4,000 pixels wide (which will be tough to view on screen) but will provide the analytical insight into how temperatures are changing over time.

The goal of these charts is to reveal any general trends towards warmer weather. Each dot on the chart represents one temperature on one day in one year and when we see clumps or clusters of these data points, we are seeing increased frequency of these temperatures. In other words, climate change inside this collection of temperature extremes. 

In general, we see more of the top 10% hottest temperatures in the 21st century and the top 10% coldest temperatures in the 19th and 20th centuries. And these charts show part of why it is so hard to detect climate change looking at daily temperatures or average temperature: Some of the hottest temperatures will appear in the 19th century and some of the coldest will show in the 21st. Our daily temperatures are determined by a big chunk of randomness, disguising the overall trends.

These charts also display "columns" of temperatures that cluster around certain years. For example, the Texas and Oklahoma charts show some of their hottest temperatures happened in the 1930s, when the dust bowl blew thousands if not millions of farmers off their land and made them factory workers or produce pickers working for someone else. I have to wonder if these high temperatures hadn't occurred during a drought if these people would not have been forced off their land.

By selecting 10% of the hottest and coldest daytime temps and nighttime temps, we can say with some confidence that 80% of the daytime and night time temperatures will between them. For example, if 10% of the hottest daytime temps for LA are above 93 degrees and 10% of the coldest daytime temps are below 51 degrees, we can assume that 80% of the days will be between 51 and 93 degrees.

A new chart is being planned, one that shows the number of weeks of in various temperature ranges by year. This will help to point out temperature trends, for example, Boston had 14 weeks where the average low temperature was below freezing in the 20th century but only 8 weeks where the average low temperature was below freezing in the 21st century.

Also planned is a "this day compared to average, median, and min-max temperatures" feature. This will require moving the calculations onto the server to allow dynamic production of the charted output, but it would be fun to know how today's temperture compares to the average temps by decade and over all temps, as well as how today compares to the median temp for that day (equal number of days higher and lower) as well as the range of temps (absolute highest and lowest) for that day. Maybe not useful in the climate change narrative, but fun to know. Hmmm, this might need to be a separate app & website. Same data but separate websites. Sister sites.