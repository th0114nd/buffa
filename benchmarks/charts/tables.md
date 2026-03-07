### Binary decode

| Message | buffa | buffa (view) | prost | protobuf-v4 | Go |
|---------|------:|------:|------:|------:|------:|
| ApiResponse | 762 | 1,245 (+63%) | 777 (+2%) | 720 (−5%) | 277 (−64%) |
| LogRecord | 689 | 1,772 (+157%) | 692 (+0%) | 882 (+28%) | 251 (−64%) |
| AnalyticsEvent | 188 | 307 (+63%) | 258 (+37%) | 364 (+93%) | 92 (−51%) |
| GoogleMessage1 | 801 | 1,093 (+36%) | 1,001 (+25%) | 659 (−18%) | 351 (−56%) |

### Binary encode

| Message | buffa | prost | protobuf-v4 | Go |
|---------|------:|------:|------:|------:|
| ApiResponse | 2,637 | 1,755 (−33%) | 1,050 (−60%) | 570 (−78%) |
| LogRecord | 4,149 | 3,163 (−24%) | 1,717 (−59%) | 309 (−93%) |
| AnalyticsEvent | 671 | 369 (−45%) | 516 (−23%) | 162 (−76%) |
| GoogleMessage1 | 2,543 | 1,866 (−27%) | 882 (−65%) | 366 (−86%) |

### JSON encode

| Message | buffa | prost | Go |
|---------|------:|------:|------:|
| ApiResponse | 869 | 776 (−11%) | 119 (−86%) |
| LogRecord | 1,335 | 1,099 (−18%) | 144 (−89%) |
| AnalyticsEvent | 781 | 768 (−2%) | 52 (−93%) |
| GoogleMessage1 | 1,047 | 840 (−20%) | 129 (−88%) |

### JSON decode

| Message | buffa | prost | Go |
|---------|------:|------:|------:|
| ApiResponse | 721 | 299 (−59%) | 71 (−90%) |
| LogRecord | 780 | 694 (−11%) | 112 (−86%) |
| AnalyticsEvent | 272 | 239 (−12%) | 47 (−83%) |
| GoogleMessage1 | 635 | 253 (−60%) | 74 (−88%) |
