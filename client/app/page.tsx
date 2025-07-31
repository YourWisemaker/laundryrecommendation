"use client"

import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import {
  Search,
  MapPin,
  Wind,
  Droplets,
  Sun,
  Cloud,
  CloudRain,
  Thermometer,
  Eye,
  Clock,
  Shirt,
  Timer,
  CheckCircle,
} from "lucide-react"

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'

interface WeatherData {
  temperature: number
  condition: string
  humidity: number
  windSpeed: number
  uvIndex: number
  chanceOfRain: number
}

interface HourlyForecast {
  time: string
  temp: number
  condition: string
  dryTime: number
}

interface WeeklyForecast {
  day: string
  dayName?: string
  condition: string
  high: number
  low: number
  dryRating: string
}

interface LocationCoords {
  lat: number
  lon: number
}

// Define types
interface CurrentWeather {
  temperature: number
  condition: string
  humidity: number
  windSpeed: number
  uvIndex: number
  chanceOfRain: number
  realFeel: number
}

export default function LaundryOptimizer() {
  const [location, setLocation] = useState('')
  const [searchValue, setSearchValue] = useState('')
  const [currentWeather, setCurrentWeather] = useState<CurrentWeather>({
    temperature: 22,
    condition: "sunny",
    humidity: 45,
    windSpeed: 0.2,
    uvIndex: 3,
    chanceOfRain: 0,
    realFeel: 25,
  })
  const [hourlyForecast, setHourlyForecast] = useState<HourlyForecast[]>([])
  const [weeklyForecast, setWeeklyForecast] = useState<WeeklyForecast[]>([])
  const [coordinates, setCoordinates] = useState<LocationCoords>({ lat: 40.4168, lon: -3.7038 }) // Default to Madrid
  const [loading, setLoading] = useState(false)
  const [recommendations, setRecommendations] = useState<any[]>([])  
  const [aiRecommendation, setAiRecommendation] = useState<string>("")  
  const [isGeneratingAI, setIsGeneratingAI] = useState(false)
  const [isUsingCurrentLocation, setIsUsingCurrentLocation] = useState(false)
  
  // Timer modal state
  const [isTimerModalOpen, setIsTimerModalOpen] = useState(false)
  const [timerHours, setTimerHours] = useState(0)
  const [timerMinutes, setTimerMinutes] = useState(30)
  const [isTimerRunning, setIsTimerRunning] = useState(false)
  const [timeRemaining, setTimeRemaining] = useState(0)
  const [timerInterval, setTimerInterval] = useState<NodeJS.Timeout | null>(null)

  // Get user's location based on IP address
  const getLocationFromIP = async () => {
    try {
      // Use a free IP geolocation service
      const response = await fetch('https://ipapi.co/json/')
      const data = await response.json()
      
      if (data.country_name) {
        // Get the capital city of the detected country
        const capital = getCountryCapital(data.country_name)
        return capital
      }
    } catch (error) {
      console.error('Failed to get location from IP:', error)
    }
    
    // Fallback to Madrid if IP detection fails
    return { name: 'Madrid, Spain', coords: { lat: 40.4168, lon: -3.7038 } }
  }

  const getCountryCapital = (countryName: string) => {
    const capitals: { [key: string]: { name: string, coords: { lat: number, lon: number } } } = {
      'United States': { name: 'Washington, D.C., United States', coords: { lat: 38.9072, lon: -77.0369 } },
      'Canada': { name: 'Ottawa, Canada', coords: { lat: 45.4215, lon: -75.6972 } },
      'United Kingdom': { name: 'London, United Kingdom', coords: { lat: 51.5074, lon: -0.1278 } },
      'France': { name: 'Paris, France', coords: { lat: 48.8566, lon: 2.3522 } },
      'Germany': { name: 'Berlin, Germany', coords: { lat: 52.5200, lon: 13.4050 } },
      'Italy': { name: 'Rome, Italy', coords: { lat: 41.9028, lon: 12.4964 } },
      'Spain': { name: 'Madrid, Spain', coords: { lat: 40.4168, lon: -3.7038 } },
      'Japan': { name: 'Tokyo, Japan', coords: { lat: 35.6762, lon: 139.6503 } },
      'China': { name: 'Beijing, China', coords: { lat: 39.9042, lon: 116.4074 } },
      'India': { name: 'New Delhi, India', coords: { lat: 28.6139, lon: 77.2090 } },
      'Australia': { name: 'Canberra, Australia', coords: { lat: -35.2809, lon: 149.1300 } },
      'Brazil': { name: 'Brasília, Brazil', coords: { lat: -15.8267, lon: -47.9218 } },
      'Mexico': { name: 'Mexico City, Mexico', coords: { lat: 19.4326, lon: -99.1332 } },
      'Russia': { name: 'Moscow, Russia', coords: { lat: 55.7558, lon: 37.6176 } },
      'South Korea': { name: 'Seoul, South Korea', coords: { lat: 37.5665, lon: 126.9780 } },
      'Netherlands': { name: 'Amsterdam, Netherlands', coords: { lat: 52.3676, lon: 4.9041 } },
      'Sweden': { name: 'Stockholm, Sweden', coords: { lat: 59.3293, lon: 18.0686 } },
      'Norway': { name: 'Oslo, Norway', coords: { lat: 59.9139, lon: 10.7522 } },
      'Denmark': { name: 'Copenhagen, Denmark', coords: { lat: 55.6761, lon: 12.5683 } },
      'Finland': { name: 'Helsinki, Finland', coords: { lat: 60.1699, lon: 24.9384 } },
      'Switzerland': { name: 'Bern, Switzerland', coords: { lat: 46.9481, lon: 7.4474 } },
      'Austria': { name: 'Vienna, Austria', coords: { lat: 48.2082, lon: 16.3738 } },
      'Belgium': { name: 'Brussels, Belgium', coords: { lat: 50.8503, lon: 4.3517 } },
      'Portugal': { name: 'Lisbon, Portugal', coords: { lat: 38.7223, lon: -9.1393 } },
      'Greece': { name: 'Athens, Greece', coords: { lat: 37.9838, lon: 23.7275 } },
      'Turkey': { name: 'Ankara, Turkey', coords: { lat: 39.9334, lon: 32.8597 } },
      'Poland': { name: 'Warsaw, Poland', coords: { lat: 52.2297, lon: 21.0122 } },
      'Czech Republic': { name: 'Prague, Czech Republic', coords: { lat: 50.0755, lon: 14.4378 } },
      'Hungary': { name: 'Budapest, Hungary', coords: { lat: 47.4979, lon: 19.0402 } },
      'Romania': { name: 'Bucharest, Romania', coords: { lat: 44.4268, lon: 26.1025 } },
      'Bulgaria': { name: 'Sofia, Bulgaria', coords: { lat: 42.6977, lon: 23.3219 } },
      'Croatia': { name: 'Zagreb, Croatia', coords: { lat: 45.8150, lon: 15.9819 } },
      'Serbia': { name: 'Belgrade, Serbia', coords: { lat: 44.7866, lon: 20.4489 } },
      'Ukraine': { name: 'Kyiv, Ukraine', coords: { lat: 50.4501, lon: 30.5234 } },
      'Belarus': { name: 'Minsk, Belarus', coords: { lat: 53.9045, lon: 27.5615 } },
      'Lithuania': { name: 'Vilnius, Lithuania', coords: { lat: 54.6872, lon: 25.2797 } },
      'Latvia': { name: 'Riga, Latvia', coords: { lat: 56.9496, lon: 24.1052 } },
      'Estonia': { name: 'Tallinn, Estonia', coords: { lat: 59.4370, lon: 24.7536 } },
      'Slovenia': { name: 'Ljubljana, Slovenia', coords: { lat: 46.0569, lon: 14.5058 } },
      'Slovakia': { name: 'Bratislava, Slovakia', coords: { lat: 48.1486, lon: 17.1077 } },
      'Ireland': { name: 'Dublin, Ireland', coords: { lat: 53.3498, lon: -6.2603 } },
      'Iceland': { name: 'Reykjavik, Iceland', coords: { lat: 64.1466, lon: -21.9426 } },
      'Luxembourg': { name: 'Luxembourg City, Luxembourg', coords: { lat: 49.6116, lon: 6.1319 } },
      'Malta': { name: 'Valletta, Malta', coords: { lat: 35.8989, lon: 14.5146 } },
      'Cyprus': { name: 'Nicosia, Cyprus', coords: { lat: 35.1856, lon: 33.3823 } },
      'Israel': { name: 'Jerusalem, Israel', coords: { lat: 31.7683, lon: 35.2137 } },
      'Saudi Arabia': { name: 'Riyadh, Saudi Arabia', coords: { lat: 24.7136, lon: 46.6753 } },
      'United Arab Emirates': { name: 'Abu Dhabi, UAE', coords: { lat: 24.2992, lon: 54.6972 } },
      'Qatar': { name: 'Doha, Qatar', coords: { lat: 25.2760, lon: 51.5200 } },
      'Kuwait': { name: 'Kuwait City, Kuwait', coords: { lat: 29.3759, lon: 47.9774 } },
      'Bahrain': { name: 'Manama, Bahrain', coords: { lat: 26.2285, lon: 50.5860 } },
      'Oman': { name: 'Muscat, Oman', coords: { lat: 23.5859, lon: 58.4059 } },
      'Jordan': { name: 'Amman, Jordan', coords: { lat: 31.9454, lon: 35.9284 } },
      'Lebanon': { name: 'Beirut, Lebanon', coords: { lat: 33.8938, lon: 35.5018 } },
      'Egypt': { name: 'Cairo, Egypt', coords: { lat: 30.0444, lon: 31.2357 } },
      'South Africa': { name: 'Cape Town, South Africa', coords: { lat: -33.9249, lon: 18.4241 } },
      'Nigeria': { name: 'Abuja, Nigeria', coords: { lat: 9.0765, lon: 7.3986 } },
      'Kenya': { name: 'Nairobi, Kenya', coords: { lat: -1.2921, lon: 36.8219 } },
      'Morocco': { name: 'Rabat, Morocco', coords: { lat: 34.0209, lon: -6.8416 } },
      'Algeria': { name: 'Algiers, Algeria', coords: { lat: 36.7538, lon: 3.0588 } },
      'Tunisia': { name: 'Tunis, Tunisia', coords: { lat: 36.8065, lon: 10.1815 } },
      'Libya': { name: 'Tripoli, Libya', coords: { lat: 32.8872, lon: 13.1913 } },
      'Ethiopia': { name: 'Addis Ababa, Ethiopia', coords: { lat: 9.1450, lon: 40.4897 } },
      'Ghana': { name: 'Accra, Ghana', coords: { lat: 5.6037, lon: -0.1870 } },
      'Senegal': { name: 'Dakar, Senegal', coords: { lat: 14.7167, lon: -17.4677 } },
      'Ivory Coast': { name: 'Yamoussoukro, Ivory Coast', coords: { lat: 6.8276, lon: -5.2893 } },
      'Cameroon': { name: 'Yaoundé, Cameroon', coords: { lat: 3.8480, lon: 11.5021 } },
      'Uganda': { name: 'Kampala, Uganda', coords: { lat: 0.3476, lon: 32.5825 } },
      'Tanzania': { name: 'Dodoma, Tanzania', coords: { lat: -6.1630, lon: 35.7516 } },
      'Zimbabwe': { name: 'Harare, Zimbabwe', coords: { lat: -17.8252, lon: 31.0335 } },
      'Zambia': { name: 'Lusaka, Zambia', coords: { lat: -15.3875, lon: 28.3228 } },
      'Botswana': { name: 'Gaborone, Botswana', coords: { lat: -24.6282, lon: 25.9231 } },
      'Namibia': { name: 'Windhoek, Namibia', coords: { lat: -22.5609, lon: 17.0658 } },
      'Madagascar': { name: 'Antananarivo, Madagascar', coords: { lat: -18.8792, lon: 47.5079 } },
      'Mauritius': { name: 'Port Louis, Mauritius', coords: { lat: -20.1609, lon: 57.5012 } },
      'Seychelles': { name: 'Victoria, Seychelles', coords: { lat: -4.6796, lon: 55.4920 } },
      'Argentina': { name: 'Buenos Aires, Argentina', coords: { lat: -34.6118, lon: -58.3960 } },
      'Chile': { name: 'Santiago, Chile', coords: { lat: -33.4489, lon: -70.6693 } },
      'Colombia': { name: 'Bogotá, Colombia', coords: { lat: 4.7110, lon: -74.0721 } },
      'Peru': { name: 'Lima, Peru', coords: { lat: -12.0464, lon: -77.0428 } },
      'Venezuela': { name: 'Caracas, Venezuela', coords: { lat: 10.4806, lon: -66.9036 } },
      'Ecuador': { name: 'Quito, Ecuador', coords: { lat: -0.1807, lon: -78.4678 } },
      'Bolivia': { name: 'Sucre, Bolivia', coords: { lat: -19.0196, lon: -65.2619 } },
      'Paraguay': { name: 'Asunción, Paraguay', coords: { lat: -25.2637, lon: -57.5759 } },
      'Uruguay': { name: 'Montevideo, Uruguay', coords: { lat: -34.9011, lon: -56.1645 } },
      'Guyana': { name: 'Georgetown, Guyana', coords: { lat: 6.8013, lon: -58.1551 } },
      'Suriname': { name: 'Paramaribo, Suriname', coords: { lat: 5.8520, lon: -55.2038 } },
      'French Guiana': { name: 'Cayenne, French Guiana', coords: { lat: 4.9346, lon: -52.3303 } },
      'Costa Rica': { name: 'San José, Costa Rica', coords: { lat: 9.9281, lon: -84.0907 } },
      'Panama': { name: 'Panama City, Panama', coords: { lat: 8.5380, lon: -80.7821 } },
      'Guatemala': { name: 'Guatemala City, Guatemala', coords: { lat: 14.6349, lon: -90.5069 } },
      'Belize': { name: 'Belmopan, Belize', coords: { lat: 17.2510, lon: -88.7590 } },
      'Honduras': { name: 'Tegucigalpa, Honduras', coords: { lat: 14.0723, lon: -87.1921 } },
      'El Salvador': { name: 'San Salvador, El Salvador', coords: { lat: 13.6929, lon: -89.2182 } },
      'Nicaragua': { name: 'Managua, Nicaragua', coords: { lat: 12.1364, lon: -86.2514 } },
      'Cuba': { name: 'Havana, Cuba', coords: { lat: 23.1136, lon: -82.3666 } },
      'Jamaica': { name: 'Kingston, Jamaica', coords: { lat: 17.9970, lon: -76.7936 } },
      'Haiti': { name: 'Port-au-Prince, Haiti', coords: { lat: 18.5944, lon: -72.3074 } },
      'Dominican Republic': { name: 'Santo Domingo, Dominican Republic', coords: { lat: 18.4861, lon: -69.9312 } },
      'Puerto Rico': { name: 'San Juan, Puerto Rico', coords: { lat: 18.4655, lon: -66.1057 } },
      'Trinidad and Tobago': { name: 'Port of Spain, Trinidad and Tobago', coords: { lat: 10.6596, lon: -61.5089 } },
      'Barbados': { name: 'Bridgetown, Barbados', coords: { lat: 13.1939, lon: -59.5432 } },
      'Bahamas': { name: 'Nassau, Bahamas', coords: { lat: 25.0443, lon: -77.3504 } },
      'Thailand': { name: 'Bangkok, Thailand', coords: { lat: 14.5995, lon: 100.5018 } },
      'Vietnam': { name: 'Hanoi, Vietnam', coords: { lat: 21.0285, lon: 105.8542 } },
      'Malaysia': { name: 'Kuala Lumpur, Malaysia', coords: { lat: 3.1390, lon: 101.6869 } },
      'Singapore': { name: 'Singapore, Singapore', coords: { lat: 1.3521, lon: 103.8198 } },
      'Indonesia': { name: 'Jakarta, Indonesia', coords: { lat: -6.2088, lon: 106.8456 } },
      'Philippines': { name: 'Manila, Philippines', coords: { lat: 14.5995, lon: 120.9842 } },
      'Cambodia': { name: 'Phnom Penh, Cambodia', coords: { lat: 11.5564, lon: 104.9282 } },
      'Laos': { name: 'Vientiane, Laos', coords: { lat: 17.9757, lon: 102.6331 } },
      'Myanmar': { name: 'Naypyidaw, Myanmar', coords: { lat: 19.7633, lon: 96.0785 } },
      'Bangladesh': { name: 'Dhaka, Bangladesh', coords: { lat: 23.8103, lon: 90.4125 } },
      'Sri Lanka': { name: 'Colombo, Sri Lanka', coords: { lat: 6.9271, lon: 79.8612 } },
      'Nepal': { name: 'Kathmandu, Nepal', coords: { lat: 27.7172, lon: 85.3240 } },
      'Bhutan': { name: 'Thimphu, Bhutan', coords: { lat: 27.4728, lon: 89.6390 } },
      'Maldives': { name: 'Malé, Maldives', coords: { lat: 4.1755, lon: 73.5093 } },
      'Pakistan': { name: 'Islamabad, Pakistan', coords: { lat: 33.6844, lon: 73.0479 } },
      'Afghanistan': { name: 'Kabul, Afghanistan', coords: { lat: 34.5553, lon: 69.2075 } },
      'Iran': { name: 'Tehran, Iran', coords: { lat: 35.6892, lon: 51.3890 } },
      'Iraq': { name: 'Baghdad, Iraq', coords: { lat: 33.3152, lon: 44.3661 } },
      'Syria': { name: 'Damascus, Syria', coords: { lat: 33.5138, lon: 36.2765 } },
      'Yemen': { name: 'Sana\'a, Yemen', coords: { lat: 15.3694, lon: 44.1910 } },
      'Kazakhstan': { name: 'Nur-Sultan, Kazakhstan', coords: { lat: 51.1694, lon: 71.4491 } },
      'Uzbekistan': { name: 'Tashkent, Uzbekistan', coords: { lat: 41.2995, lon: 69.2401 } },
      'Turkmenistan': { name: 'Ashgabat, Turkmenistan', coords: { lat: 37.9601, lon: 58.3261 } },
      'Kyrgyzstan': { name: 'Bishkek, Kyrgyzstan', coords: { lat: 42.8746, lon: 74.5698 } },
      'Tajikistan': { name: 'Dushanbe, Tajikistan', coords: { lat: 38.5598, lon: 68.7870 } },
      'Mongolia': { name: 'Ulaanbaatar, Mongolia', coords: { lat: 47.8864, lon: 106.9057 } },
      'North Korea': { name: 'Pyongyang, North Korea', coords: { lat: 39.0392, lon: 125.7625 } },
      'Taiwan': { name: 'Taipei, Taiwan', coords: { lat: 25.0330, lon: 121.5654 } },
      'Hong Kong': { name: 'Hong Kong, Hong Kong', coords: { lat: 22.3193, lon: 114.1694 } },
      'Macau': { name: 'Macau, Macau', coords: { lat: 22.1987, lon: 113.5439 } },
      'New Zealand': { name: 'Wellington, New Zealand', coords: { lat: -41.2924, lon: 174.7787 } },
      'Fiji': { name: 'Suva, Fiji', coords: { lat: -18.1248, lon: 178.4501 } },
      'Papua New Guinea': { name: 'Port Moresby, Papua New Guinea', coords: { lat: -9.4438, lon: 147.1803 } },
      'Solomon Islands': { name: 'Honiara, Solomon Islands', coords: { lat: -9.4280, lon: 159.9729 } },
      'Vanuatu': { name: 'Port Vila, Vanuatu', coords: { lat: -17.7334, lon: 168.3273 } },
      'Samoa': { name: 'Apia, Samoa', coords: { lat: -13.8506, lon: -171.7513 } },
      'Tonga': { name: 'Nuku\'alofa, Tonga', coords: { lat: -21.1789, lon: -175.1982 } },
      'Palau': { name: 'Ngerulmud, Palau', coords: { lat: 7.5006, lon: 134.6242 } },
      'Marshall Islands': { name: 'Majuro, Marshall Islands', coords: { lat: 7.1315, lon: 171.1845 } },
      'Micronesia': { name: 'Palikir, Micronesia', coords: { lat: 6.9248, lon: 158.1611 } },
      'Kiribati': { name: 'Tarawa, Kiribati', coords: { lat: 1.3278, lon: 172.9779 } },
      'Tuvalu': { name: 'Funafuti, Tuvalu', coords: { lat: -8.5243, lon: 179.1942 } },
      'Nauru': { name: 'Yaren, Nauru', coords: { lat: -0.5477, lon: 166.9209 } }
    }
    
    return capitals[countryName] || { name: 'Madrid, Spain', coords: { lat: 40.4168, lon: -3.7038 } }
  }

  // Find nearest big city or province city
  const findNearestBigCity = async (lat: number, lon: number) => {
    try {
      // Search for nearby cities with a larger radius to find major cities
      const response = await fetch(`${API_BASE_URL}/geocode?lat=${lat}&lon=${lon}&limit=10`)
      if (response.ok) {
        const data = await response.json()
        
        // Filter for cities with population > 100,000 or administrative importance
        const bigCities = data.filter((place: any) => 
          place.feature_code === 'PPLA' || // seat of a first-order administrative division
          place.feature_code === 'PPLA2' || // seat of a second-order administrative division
          place.feature_code === 'PPLC' || // capital of a political entity
          (place.population && place.population > 100000) // cities with population > 100k
        )
        
        if (bigCities.length > 0) {
          const nearestCity = bigCities[0]
          const locationName = nearestCity.state 
            ? `${nearestCity.name}, ${nearestCity.state}, ${nearestCity.country}`
            : `${nearestCity.name}, ${nearestCity.country}`
          return {
            name: locationName,
            coords: { lat: nearestCity.lat, lon: nearestCity.lon }
          }
        }
        
        // If no big cities found, try to find the nearest city with any population data
        const citiesWithPopulation = data.filter((place: any) => place.population && place.population > 50000)
        if (citiesWithPopulation.length > 0) {
          const nearestCity = citiesWithPopulation[0]
          const locationName = nearestCity.state 
            ? `${nearestCity.name}, ${nearestCity.state}, ${nearestCity.country}`
            : `${nearestCity.name}, ${nearestCity.country}`
          return {
            name: locationName,
            coords: { lat: nearestCity.lat, lon: nearestCity.lon }
          }
        }
      }
    } catch (error) {
      console.error('Error finding nearest big city:', error)
    }
    return null
  }

  // Initialize default location based on user's IP
  useEffect(() => {
    const initializeLocation = async () => {
      const defaultLocation = await getLocationFromIP()
      setLocation(defaultLocation.name)
      setCoordinates(defaultLocation.coords)
      fetchWeatherData(defaultLocation.coords)
    }
    initializeLocation()
  }, [])

  useEffect(() => {
    if (currentWeather.temperature && !aiRecommendation) {
      generateAIRecommendation()
    }
  }, [currentWeather.temperature])

  // Fetch geocoding data
  const geocodeLocation = async (locationName: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/geocode?q=${encodeURIComponent(locationName)}&limit=1`)
      if (response.ok) {
        const data = await response.json()
        if (data.length > 0) {
          const location = data[0]
          const displayName = location.state 
            ? `${location.name}, ${location.state}, ${location.country}`
            : `${location.name}, ${location.country}`
          setLocation(displayName)
          setCoordinates({ lat: location.lat, lon: location.lon })
          return { lat: location.lat, lon: location.lon }
        }
      }
    } catch (error) {
      console.error('Geocoding error:', error)
    }
    return null
  }

  // Fetch weather data and recommendations
  const fetchWeatherData = async (coords: LocationCoords) => {
    setLoading(true)
    try {
      // Get drying windows/recommendations
      const windowsResponse = await fetch(
        `${API_BASE_URL}/drying-windows?lat=${coords.lat}&lon=${coords.lon}&hours=24`
      )
      
      if (windowsResponse.ok) {
        const windowsData = await windowsResponse.json()
        
        // Transform the data for our UI
        if (windowsData.windows && windowsData.windows.length > 0) {
          const firstWindow = windowsData.windows[0]
          
          // Update current weather from the first window's weather summary
          if (firstWindow.weather_summary) {
            const summary = firstWindow.weather_summary
            setCurrentWeather({
              temperature: Math.round(summary.avg_temp_c),
              condition: summary.conditions?.toLowerCase() || "sunny",
              humidity: summary.avg_humidity,
              windSpeed: summary.avg_wind_ms * 3.6, // Convert m/s to km/h
              uvIndex: 3, // Default UV index
              chanceOfRain: summary.total_rain_mm > 0 ? 80 : 0,
              realFeel: Math.round(summary.avg_temp_c + 2), // Estimate real feel
            })
          }
          
          // Transform hourly data from windows
          const hourlyData: HourlyForecast[] = windowsData.windows.slice(0, 6).map((window: any, index: number) => {
            const startTime = new Date(window.start_time)
            const dryTimeHours = Math.max(1, Math.round(4 - (window.score?.score || 0.5) * 3)) // Estimate based on score
            
            return {
              time: startTime.toLocaleTimeString('en-US', { hour: 'numeric', hour12: true }),
              temp: Math.round(window.weather_summary?.avg_temp_c || 25),
              condition: window.weather_summary?.conditions?.toLowerCase() || "sunny",
              dryTime: dryTimeHours,
            }
          })
          setHourlyForecast(hourlyData)
        }
      }
      
      // Get recommendations for 7-day forecast
      const recsResponse = await fetch(
        `${API_BASE_URL}/recommendations?lat=${coords.lat}&lon=${coords.lon}&hours=168&limit=20`
      )
      
      if (recsResponse.ok) {
        const recsData = await recsResponse.json()
        setRecommendations(recsData.best_windows || [])
        
        // Create weekly forecast from best windows - extend to 7 days
        const weekly: WeeklyForecast[] = []
        const days = ['Today', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
        
        // Use backend data for available windows
        const availableWindows = recsData.best_windows || []
        
        for (let i = 0; i < 7; i++) {
          if (i < availableWindows.length) {
            // Use real backend data
            const window = availableWindows[i]
            const score = window.score?.score || 0.5
            const temp = window.weather_summary?.avg_temp_c || 25
            
            const forecastDate = new Date()
            forecastDate.setDate(forecastDate.getDate() + i)
            const dateStr = forecastDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
            const dayName = forecastDate.toLocaleDateString('en-US', { weekday: 'long' })
            
            weekly.push({
              day: i === 0 ? 'Today' : dateStr,
              dayName: i === 0 ? 'Today' : dayName,
              condition: window.weather_summary?.conditions?.toLowerCase() || "sunny",
              high: Math.round(temp + 5),
              low: Math.round(temp - 5),
              dryRating: score > 0.8 ? "excellent" : score > 0.6 ? "good" : score > 0.4 ? "fair" : "poor",
            })
          } else {
            // Generate forecast for remaining days using pattern from available data
            const lastWindow = availableWindows[availableWindows.length - 1]
            const baseTemp = lastWindow?.weather_summary?.avg_temp_c || 25
            const tempVariation = Math.random() * 6 - 3 // ±3°C variation
            const finalTemp = Math.round(baseTemp + tempVariation)
            
            // Simulate weather patterns
            const conditions = ['sunny', 'cloudy', 'partly-cloudy']
            const randomCondition = conditions[Math.floor(Math.random() * conditions.length)]
            const randomScore = 0.3 + Math.random() * 0.5 // Score between 0.3-0.8
            
            const forecastDate = new Date()
            forecastDate.setDate(forecastDate.getDate() + i)
            const dateStr = forecastDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
            const dayName = forecastDate.toLocaleDateString('en-US', { weekday: 'long' })
            
            weekly.push({
              day: i === 0 ? 'Today' : dateStr,
              dayName: i === 0 ? 'Today' : dayName,
              condition: randomCondition,
              high: Math.round(finalTemp + 5),
              low: Math.round(finalTemp - 5),
              dryRating: randomScore > 0.8 ? "excellent" : randomScore > 0.6 ? "good" : randomScore > 0.4 ? "fair" : "poor",
            })
          }
        }
        
        setWeeklyForecast(weekly)
      }
      
    } catch (error) {
      console.error('Weather data fetch error:', error)
    } finally {
      setLoading(false)
    }
  }

  // Initial load
  useEffect(() => {
    fetchWeatherData(coordinates)
  }, [coordinates])

  // Handle location search
  const handleLocationSearch = async () => {
    if (searchValue.trim()) {
      const coords = await geocodeLocation(searchValue.trim())
      if (coords) {
        setLocation(searchValue.trim())
        setCoordinates(coords)
        setIsUsingCurrentLocation(false)
      }
    }
  }

  // Handle current location detection
  const handleUseCurrentLocation = () => {
    if (!navigator.geolocation) {
      alert('Geolocation is not supported by this browser.')
      return
    }

    setLoading(true)
    setIsUsingCurrentLocation(true)

    navigator.geolocation.getCurrentPosition(
      async (position) => {
        const { latitude, longitude } = position.coords
        const newCoords = { lat: latitude, lon: longitude }
        
        // Find nearest big city or province city
        try {
          const nearestCity = await findNearestBigCity(latitude, longitude)
          if (nearestCity) {
            setLocation(nearestCity.name)
            setCoordinates(nearestCity.coords)
          } else {
            // Fallback to reverse geocoding
            const response = await fetch(`${API_BASE_URL}/geocode?lat=${latitude}&lon=${longitude}&limit=1`)
            if (response.ok) {
              const data = await response.json()
              if (data.length > 0) {
                const location = data[0]
                const locationName = location.state 
                  ? `${location.name}, ${location.state}, ${location.country}`
                  : `${location.name}, ${location.country}`
                setLocation(locationName)
              } else {
                setLocation(`${latitude.toFixed(4)}, ${longitude.toFixed(4)}`)
              }
            } else {
              setLocation(`${latitude.toFixed(4)}, ${longitude.toFixed(4)}`)
            }
          }
        } catch (error) {
          console.error('Location detection error:', error)
          setLocation(`${latitude.toFixed(4)}, ${longitude.toFixed(4)}`)
        }
        
        setCoordinates(newCoords)
        setLoading(false)
      },
      (error) => {
        console.error('Geolocation error:', error)
        alert('Unable to retrieve your location. Please try again or search manually.')
        setLoading(false)
        setIsUsingCurrentLocation(false)
      },
      {
        enableHighAccuracy: true,
        timeout: 10000,
        maximumAge: 300000 // 5 minutes
      }
    )
  }

  // Timer functions
  const startTimer = () => {
    const totalMinutes = timerHours * 60 + timerMinutes
    if (totalMinutes <= 0) return
    
    setTimeRemaining(totalMinutes * 60) // Convert to seconds
    setIsTimerRunning(true)
    setIsTimerModalOpen(false)
    
    const interval = setInterval(() => {
      setTimeRemaining(prev => {
        if (prev <= 1) {
          // Timer finished
          setIsTimerRunning(false)
          clearInterval(interval)
          setTimerInterval(null)
          playAlarm()
          alert('Drying time is up!')
          return 0
        }
        return prev - 1
      })
    }, 1000)
    
    setTimerInterval(interval)
  }
  
  const stopTimer = () => {
    if (timerInterval) {
      clearInterval(timerInterval)
      setTimerInterval(null)
    }
    setIsTimerRunning(false)
    setTimeRemaining(0)
  }
  
  const playAlarm = () => {
    // Try to play alarm.mp3 first
    const audio = new Audio('/alarm.mp3')
    audio.play().catch(error => {
      console.log('Could not play alarm.mp3, generating beep sound:', error)
      // Generate beep sound using Web Audio API
      try {
        const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
        const oscillator = audioContext.createOscillator()
        const gainNode = audioContext.createGain()
        
        oscillator.connect(gainNode)
        gainNode.connect(audioContext.destination)
        
        oscillator.frequency.value = 800 // 800 Hz tone
        oscillator.type = 'sine'
        
        gainNode.gain.setValueAtTime(0.3, audioContext.currentTime)
        gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 1)
        
        oscillator.start(audioContext.currentTime)
        oscillator.stop(audioContext.currentTime + 1)
        
        // Play multiple beeps
        setTimeout(() => {
          const oscillator2 = audioContext.createOscillator()
          const gainNode2 = audioContext.createGain()
          oscillator2.connect(gainNode2)
          gainNode2.connect(audioContext.destination)
          oscillator2.frequency.value = 1000
          oscillator2.type = 'sine'
          gainNode2.gain.setValueAtTime(0.3, audioContext.currentTime)
          gainNode2.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 1)
          oscillator2.start(audioContext.currentTime)
          oscillator2.stop(audioContext.currentTime + 1)
        }, 500)
        
      } catch (audioError) {
        console.log('Web Audio API not supported, using speech synthesis:', audioError)
        // Final fallback to speech synthesis
        if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
          const utterance = new SpeechSynthesisUtterance('Drying time is up!')
          utterance.rate = 1.2
          utterance.pitch = 1.5
          speechSynthesis.speak(utterance)
        }
      }
    })
  }
  
  const formatTime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60
    
    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`
  }

  const generateAIRecommendation = async () => {
    setIsGeneratingAI(true)
    try {
      const response = await fetch(`${API_BASE_URL}/ai-recommendation?lat=${coordinates.lat}&lon=${coordinates.lon}`)

      if (!response.ok) {
        if (response.status === 429) {
          setAiRecommendation('AI service is currently busy due to high demand. Please try again in a few moments. In the meantime, check the weather conditions above for guidance!')
        } else if (response.status >= 500) {
          setAiRecommendation('AI service is temporarily unavailable. Based on current weather conditions, consider drying your laundry accordingly.')
        } else {
          throw new Error(`HTTP ${response.status}: Failed to generate AI recommendation`)
        }
        return
      }

      const data = await response.json()
      setAiRecommendation(data.recommendation)
    } catch (error) {
      console.error('Error generating AI recommendation:', error)
      setAiRecommendation('Unable to connect to AI service. Weather looks good for laundry! Consider the current conditions and dry accordingly.')
    } finally {
      setIsGeneratingAI(false)
    }
  }

  // Mock data fallbacks
  const defaultHourlyForecast = [
    { time: "6:00 AM", temp: 25, condition: "cloudy", dryTime: 4 },
    { time: "9:00 AM", temp: 28, condition: "sunny", dryTime: 2.5 },
    { time: "12:00 PM", temp: 33, condition: "sunny", dryTime: 2 },
    { time: "3:00 PM", temp: 34, condition: "sunny", dryTime: 1.5 },
    { time: "6:00 PM", temp: 32, condition: "sunny", dryTime: 2 },
    { time: "9:00 PM", temp: 30, condition: "cloudy", dryTime: 3 },
  ]

  const defaultWeeklyForecast = [
    { day: "Today", dayName: "Today", condition: "sunny", high: 36, low: 22, dryRating: "excellent" },
    { day: "Aug 1, 2025", dayName: "Tuesday", condition: "sunny", high: 37, low: 21, dryRating: "excellent" },
    { day: "Aug 2, 2025", dayName: "Wednesday", condition: "sunny", high: 37, low: 21, dryRating: "excellent" },
    { day: "Aug 3, 2025", dayName: "Thursday", condition: "cloudy", high: 37, low: 21, dryRating: "good" },
    { day: "Aug 4, 2025", dayName: "Friday", condition: "cloudy", high: 37, low: 21, dryRating: "good" },
    { day: "Aug 5, 2025", dayName: "Saturday", condition: "rainy", high: 37, low: 21, dryRating: "poor" },
    { day: "Aug 6, 2025", dayName: "Sunday", condition: "storm", high: 37, low: 21, dryRating: "avoid" },
  ]

  const getWeatherIcon = (condition: string, size = 24) => {
    switch (condition) {
      case "sunny":
        return <Sun size={size} className="text-yellow-400" />
      case "cloudy":
        return <Cloud size={size} className="text-gray-400" />
      case "rainy":
        return <CloudRain size={size} className="text-blue-400" />
      case "storm":
        return <CloudRain size={size} className="text-purple-400" />
      default:
        return <Sun size={size} className="text-yellow-400" />
    }
  }

  const getDryRatingColor = (rating: string) => {
    switch (rating) {
      case "excellent":
        return "bg-green-500"
      case "good":
        return "bg-yellow-500"
      case "poor":
        return "bg-orange-500"
      case "avoid":
        return "bg-red-500"
      default:
        return "bg-gray-500"
    }
  }

  const getRecommendation = () => {
    if (currentWeather.chanceOfRain === 0 && currentWeather.temperature > 25) {
      return {
        status: "excellent",
        message: "Perfect conditions for outdoor drying!",
        estimatedTime: "2-3 hours",
        tips: ["Hang clothes in direct sunlight", "Use clothespins to prevent wind damage", "Check clothes every hour"],
      }
    }
    return {
      status: "good",
      message: "Good conditions for drying",
      estimatedTime: "3-4 hours",
      tips: ["Consider partial shade", "Monitor weather changes"],
    }
  }

  const recommendation = getRecommendation()

  return (
    <div className="min-h-screen bg-slate-900 text-white">
      <div className="p-6">
        {/* Add title */}
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-white mb-2">Laundry Recommendation</h1>
          <p className="text-gray-400">AI-powered weather-based laundry drying optimization</p>
        </div>

        {/* Search Bar */}
        <div className="relative mb-8">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
          <Input
            placeholder="Search for cities"
            value={searchValue}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearchValue(e.target.value)}
            onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => {
              if (e.key === 'Enter') {
                handleLocationSearch()
              }
            }}
            className="pl-10 bg-slate-800 border-slate-700 text-white placeholder-gray-400"
          />
          {loading && (
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
              <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
            </div>
          )}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Current Weather & AI Recommendation */}
          <div className="lg:col-span-2 space-y-6">
            {/* Current Location & Weather */}
            <Card className="bg-slate-800 border-slate-700">
              <CardContent className="p-6">
                <div className="flex justify-between items-start mb-6">
                  <div>
                    <h1 className="text-3xl font-bold text-white mb-2">{location}</h1>
                    <p className="text-gray-400">Chance of rain: {currentWeather.chanceOfRain}%</p>
                    <p className="text-gray-400 text-sm">{new Date().toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}</p>
                  </div>
                  <div className="text-right">
                    <div className="text-6xl font-bold text-white mb-2">{currentWeather.temperature}°</div>
                    <div className="flex justify-end">{getWeatherIcon("sunny", 64)}</div>
                  </div>
                </div>

                {/* AI Recommendation */}
                <div className="bg-slate-700 rounded-lg p-4 mb-6">
                  <div className="flex items-center space-x-2 mb-3">
                    <CheckCircle className="w-5 h-5 text-green-400" />
                    <h3 className="text-lg font-semibold text-white">AI Laundry Recommendation</h3>
                  </div>
                  {aiRecommendation ? (
                    <p className="text-green-400 font-medium mb-2">{aiRecommendation}</p>
                  ) : (
                    <p className="text-gray-400 font-medium mb-2">Click "Get AI Recommendation" to get personalized laundry advice based on current weather conditions.</p>
                  )}
                  <div className="flex items-center space-x-4 text-sm text-gray-300">
                    <div className="flex items-center space-x-1">
                      <Clock className="w-4 h-4" />
                      <span>Est. dry time: {recommendation.estimatedTime}</span>
                    </div>
                    <Badge className={`${getDryRatingColor("excellent")} text-white`}>Excellent Conditions</Badge>
                  </div>
                </div>

                {/* Today's Drying Forecast */}
                <div>
                  <h3 className="text-lg font-semibold text-white mb-4">TODAY'S DRYING FORECAST</h3>
                  <div className="grid grid-cols-6 gap-4">
                    {(hourlyForecast.length > 0 ? hourlyForecast : defaultHourlyForecast).map((hour, index) => (
                      <div key={index} className="text-center">
                        <p className="text-xs text-gray-400 mb-2">{hour.time}</p>
                        <div className="flex justify-center mb-2">{getWeatherIcon(hour.condition, 32)}</div>
                        <p className="text-white font-semibold mb-1">{hour.temp}°</p>
                        <p className="text-xs text-blue-400">{hour.dryTime}h dry</p>
                      </div>
                    ))}
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Air Conditions */}
            <Card className="bg-slate-800 border-slate-700">
              <CardContent className="p-6">
                <div className="flex justify-between items-center mb-4">
                  <h3 className="text-lg font-semibold text-white">DRYING CONDITIONS</h3>

                </div>
                <div className="grid grid-cols-2 gap-6">
                  <div className="flex items-center space-x-3">
                    <Thermometer className="w-5 h-5 text-gray-400" />
                    <div>
                      <p className="text-gray-400 text-sm">Real Feel</p>
                      <p className="text-white text-xl font-semibold">{currentWeather.realFeel}°</p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-3">
                    <Wind className="w-5 h-5 text-gray-400" />
                    <div>
                      <p className="text-gray-400 text-sm">Wind</p>
                      <p className="text-white text-xl font-semibold">{currentWeather.windSpeed.toFixed(2)} km/h</p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-3">
                    <Droplets className="w-5 h-5 text-gray-400" />
                    <div>
                      <p className="text-gray-400 text-sm">Humidity</p>
                      <p className="text-white text-xl font-semibold">{currentWeather.humidity.toFixed(2)}%</p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-3">
                    <Eye className="w-5 h-5 text-gray-400" />
                    <div>
                      <p className="text-gray-400 text-sm">UV Index</p>
                      <p className="text-white text-xl font-semibold">{currentWeather.uvIndex}</p>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* 7-Day Drying Forecast */}
          <div>
            <Card className="bg-slate-800 border-slate-700">
              <CardContent className="p-6">
                <h3 className="text-lg font-semibold text-white mb-4">7-DAY DRYING FORECAST</h3>
                <div className="space-y-4">
                  {(weeklyForecast.length > 0 ? weeklyForecast : defaultWeeklyForecast).map((day, index) => (
                    <div key={index} className="flex items-center justify-between">
                      <div className="flex items-center space-x-3 flex-1">
                        <div className="w-20">
                          <div className="text-white text-sm font-medium">{day.dayName || day.day}</div>
                          <div className="text-gray-400 text-xs">{day.day !== day.dayName ? day.day : ''}</div>
                        </div>
                        <div className="flex items-center justify-center space-x-4 flex-1">
                          <div className="flex items-center space-x-2">
                            {getWeatherIcon(day.condition, 20)}
                            <span className="text-white text-sm capitalize">{day.condition}</span>
                          </div>
                          <Badge className={`${getDryRatingColor(day.dryRating)} text-white text-xs px-2 py-1`}>
                            {day.dryRating}
                          </Badge>
                        </div>
                        <div className="w-16 text-right">
                          <span className="text-gray-400 text-sm">
                            {day.high}/{day.low}
                          </span>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Quick Actions */}
            <Card className="bg-slate-800 border-slate-700 mt-6">
              <CardContent className="p-6">
                <h3 className="text-lg font-semibold text-white mb-4">QUICK ACTIONS</h3>
                <div className="space-y-3">
                  <Button 
                    className="w-full bg-blue-600 hover:bg-blue-700 text-white disabled:opacity-50"
                    onClick={handleUseCurrentLocation}
                    disabled={loading}
                  >
                    {loading && isUsingCurrentLocation ? (
                      <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                    ) : (
                      <MapPin className="w-4 h-4 mr-2" />
                    )}
                    {loading && isUsingCurrentLocation ? 'Detecting Location...' : 'Use Current Location'}
                  </Button>
                  <Button 
                    className="w-full bg-green-600 hover:bg-green-700 text-white disabled:opacity-50"
                    onClick={generateAIRecommendation}
                    disabled={isGeneratingAI}
                  >
                    {isGeneratingAI ? (
                      <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                    ) : (
                      <Shirt className="w-4 h-4 mr-2" />
                    )}
                    {isGeneratingAI ? 'Generating AI Recommendation...' : 'Get AI Recommendation'}
                  </Button>
                  <Dialog open={isTimerModalOpen} onOpenChange={setIsTimerModalOpen}>
                    <DialogTrigger asChild>
                      <Button
                        variant="outline"
                        className="w-full border-slate-600 text-gray-300 hover:bg-slate-700 bg-transparent"
                      >
                        <Timer className="w-4 h-4 mr-2" />
                        {isTimerRunning ? `Timer: ${formatTime(timeRemaining)}` : 'Set Drying Timer'}
                      </Button>
                    </DialogTrigger>
                    <DialogContent className="bg-slate-800 border-slate-700 text-white">
                      <DialogHeader>
                        <DialogTitle>Set Drying Timer</DialogTitle>
                      </DialogHeader>
                      <div className="space-y-4">
                        <div className="grid grid-cols-2 gap-4">
                          <div>
                            <label className="block text-sm font-medium mb-2">Hours</label>
                            <Input
                              type="number"
                              min="0"
                              max="23"
                              value={timerHours}
                              onChange={(e) => setTimerHours(parseInt(e.target.value) || 0)}
                              className="bg-slate-700 border-slate-600 text-white"
                            />
                          </div>
                          <div>
                            <label className="block text-sm font-medium mb-2">Minutes</label>
                            <Input
                              type="number"
                              min="0"
                              max="59"
                              value={timerMinutes}
                              onChange={(e) => setTimerMinutes(parseInt(e.target.value) || 0)}
                              className="bg-slate-700 border-slate-600 text-white"
                            />
                          </div>
                        </div>
                        <div className="flex gap-2">
                          <Button
                            onClick={startTimer}
                            className="flex-1 bg-green-600 hover:bg-green-700"
                            disabled={timerHours === 0 && timerMinutes === 0}
                          >
                            Start Timer
                          </Button>
                          {isTimerRunning && (
                            <Button
                              onClick={stopTimer}
                              variant="outline"
                              className="flex-1 border-red-600 text-red-400 hover:bg-red-600 hover:text-white"
                            >
                              Stop Timer
                            </Button>
                          )}
                        </div>
                        {isTimerRunning && (
                          <div className="text-center p-4 bg-slate-700 rounded-lg">
                            <div className="text-2xl font-bold text-green-400">
                              {formatTime(timeRemaining)}
                            </div>
                            <div className="text-sm text-gray-400">Time Remaining</div>
                          </div>
                        )}
                      </div>
                    </DialogContent>
                  </Dialog>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  )
}
