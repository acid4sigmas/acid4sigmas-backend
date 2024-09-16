## Cloudthemes 
the cloudthemes api point makes it possible to sync your theme with the cloud servers

**base endpoint**: /api/cloudthemes
**requires token**: **YES**
**requires verified email**: **YES**


### GET - /api/cloudthemes
**method**: GET
**required headers**: Authorization: yourtoken
**description**: returns stored cloudthemes if any present
**example request in javascript**
```js
const fetchCloudThemes = async () => {
    const token = "yourtoken"; // Replace with your token

    try {
        const response = await fetch('http://127.0.0.1:8080/api/cloudthemes', {
            method: 'GET',
            headers: {
                'Authorization': token,  // Authorization header with the token
                'Content-Type': 'application/json'  
            }
        });

        if (!response.ok) {
            throw new Error(`Error: ${response.status}`);
        }

        const data = await response.json(); // will return a json 
        console.log('Cloud Themes:', data);
    } catch (error) {
        console.error('Failed to fetch cloud themes:', error);
    }
};

fetchCloudThemes();
```

**possible status codes** 
- 200
- 403
- 404
- 500

please check error message for the status code you receive.