import os
import requests


def download_subpath(url, subpath, output_folder):
    # 创建目标文件夹
    if not os.path.exists(output_folder):
        os.makedirs(output_folder)

    # 发起HTTP请求获取子路径内容
    response = requests.get(url + subpath)

    if response.status_code == 200:
        # 提取子路径的名称作为保存文件名
        subpath_name = subpath.lstrip('/').replace('/', '_')
        output_path = os.path.join(output_folder, f'{subpath_name}.html')

        # 将内容保存到本地文件
        with open(output_path, 'wb') as file:
            file.write(response.content)

        print(f'Content downloaded and saved to: {output_path}')
    else:
        print(f'Failed to download content for subpath: {subpath}')


if __name__ == "__main__":
    base_url = "https://cloud.baidu.com/"
    subpath_to_download = "/product/a"
    output_folder = "downloaded_content"

    download_subpath(base_url, subpath_to_download, output_folder)
