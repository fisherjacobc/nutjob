# Nutjob

A rust service designed to work with NUT (Network UPS Tools) to automatically reboot computers/devices after a power outage.


## Features

- Tracks devices via ping
    - Only wakes devices that were online before the UPS switched to battery
- Supports NUT (Network UPS Tools) to get information about the attached UPS
- Persisting state file in case the nutjob service stops early (such as losing power)
- Supports resolvable hostnames and ARP for pulling MAC addresses
## Deployment

### Configuration

Make sure you have your configuration file filled out. You can copy the [example](/example.config.yaml) and modify properties to your liking.

You can easily deploy this as a docker container

```bash
docker run -d \
  --name nutjob \
  --restart unless-stopped \
  -v ~/nutjob-config.yaml:/nutjob/config.yaml \
  --network host \
  fisherjacobc/nutjob:latest
```


## License

[MIT](/LICENSE)


## Contributing

Contributions are always welcome!

Just make a pull request (or if you find something wrong, also make an issue)!