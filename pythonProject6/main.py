import requests

url = input("url:")
file_name = input('name:')
custom_filename = file_name + '.pdf'

download_path = '/Users/xuzicheng/Downloads/'

response = requests.get(url,verify=False)

# 构建自定义文件的完整路径
custom_filepath = download_path + '/' + custom_filename

with open(custom_filepath, 'wb') as f:
    f.write(response.content)
